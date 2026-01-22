//! Consensus Service
//!
//! Implements a pBFT-inspired consensus mechanism for reviewing swarm execution results.
//! Multiple reviewer agents analyze the combined changes and vote on whether to approve
//! or reject the merge.

use db::models::{
    agent_profile::AgentProfile,
    consensus_review::{
        ConsensusReview, ConsensusSummary, ConsensusVote, CreateConsensusReview, SubmitReviewVote,
    },
    swarm_execution::{SwarmExecution, SwarmExecutionStatus},
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ConsensusError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Execution not found: {0}")]
    ExecutionNotFound(Uuid),
    #[error("Review not found: {0}")]
    ReviewNotFound(Uuid),
    #[error("Not in review phase")]
    NotInReviewPhase,
    #[error("No reviewers available")]
    NoReviewersAvailable,
    #[error("Consensus not reached")]
    ConsensusNotReached,
    #[error("Review already submitted")]
    ReviewAlreadySubmitted,
}

/// Result of consensus evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusResult {
    /// Consensus reached - approved for merge
    Approved,
    /// Consensus reached - rejected
    Rejected { reasons: Vec<String> },
    /// Voting still in progress
    Pending { summary: ConsensusSummary },
    /// Requires human intervention
    Deadlock { summary: ConsensusSummary },
}

/// Configuration for consensus service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Minimum number of reviewers
    pub min_reviewers: i32,
    /// Maximum rounds before escalation
    pub max_rounds: i32,
    /// Time limit per review in seconds
    pub review_timeout_seconds: i64,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            min_reviewers: 3,
            max_rounds: 3,
            review_timeout_seconds: 1800, // 30 minutes
        }
    }
}

/// Service for managing pBFT-style consensus reviews
pub struct ConsensusService {
    pool: SqlitePool,
    config: ConsensusConfig,
}

impl ConsensusService {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            config: ConsensusConfig::default(),
        }
    }

    pub fn with_config(pool: SqlitePool, config: ConsensusConfig) -> Self {
        Self { pool, config }
    }

    /// Start the consensus review process for a swarm execution
    pub async fn start_review(
        &self,
        swarm_execution_id: Uuid,
    ) -> Result<Vec<ConsensusReview>, ConsensusError> {
        let execution = SwarmExecution::find_by_id(&self.pool, swarm_execution_id)
            .await?
            .ok_or(ConsensusError::ExecutionNotFound(swarm_execution_id))?;

        if execution.status != SwarmExecutionStatus::Reviewing {
            return Err(ConsensusError::NotInReviewPhase);
        }

        // Get reviewer agents
        let reviewers = AgentProfile::find_reviewers(&self.pool).await?;
        
        if reviewers.len() < self.config.min_reviewers as usize {
            return Err(ConsensusError::NoReviewersAvailable);
        }

        // Get current round
        let current_round = ConsensusReview::get_latest_round(&self.pool, swarm_execution_id).await?;
        let new_round = current_round + 1;

        // Create review entries for each reviewer
        let mut reviews = Vec::new();
        let reviewer_count = execution.reviewer_count.min(reviewers.len() as i32);

        for reviewer in reviewers.into_iter().take(reviewer_count as usize) {
            let review = ConsensusReview::create(
                &self.pool,
                &CreateConsensusReview {
                    swarm_execution_id,
                    reviewer_profile_id: reviewer.id,
                    round: Some(new_round),
                },
            )
            .await?;
            reviews.push(review);
        }

        Ok(reviews)
    }

    /// Submit a review vote
    pub async fn submit_vote(
        &self,
        review_id: Uuid,
        vote_data: &SubmitReviewVote,
    ) -> Result<ConsensusReview, ConsensusError> {
        let existing_review = ConsensusReview::find_by_id(&self.pool, review_id)
            .await?
            .ok_or(ConsensusError::ReviewNotFound(review_id))?;

        if existing_review.vote != ConsensusVote::Pending {
            return Err(ConsensusError::ReviewAlreadySubmitted);
        }

        let review = ConsensusReview::submit_vote(&self.pool, review_id, vote_data).await?;

        // Update swarm execution consensus counts
        match vote_data.vote {
            ConsensusVote::Approve => {
                SwarmExecution::increment_approval(&self.pool, existing_review.swarm_execution_id)
                    .await?;
            }
            ConsensusVote::Reject => {
                SwarmExecution::increment_rejection(&self.pool, existing_review.swarm_execution_id)
                    .await?;
            }
            _ => {}
        }

        Ok(review)
    }

    /// Evaluate the current consensus state
    pub async fn evaluate_consensus(
        &self,
        swarm_execution_id: Uuid,
    ) -> Result<ConsensusResult, ConsensusError> {
        let execution = SwarmExecution::find_by_id(&self.pool, swarm_execution_id)
            .await?
            .ok_or(ConsensusError::ExecutionNotFound(swarm_execution_id))?;

        let summary = ConsensusReview::get_consensus_summary(
            &self.pool,
            swarm_execution_id,
            execution.consensus_threshold,
        )
        .await?;

        // Check if consensus is reached
        if summary.has_consensus {
            return Ok(ConsensusResult::Approved);
        }

        if summary.consensus_failed {
            // Gather rejection reasons
            let reviews =
                ConsensusReview::find_by_swarm_execution(&self.pool, swarm_execution_id).await?;
            let reasons: Vec<String> = reviews
                .into_iter()
                .filter(|r| r.vote == ConsensusVote::Reject)
                .filter_map(|r| r.comments)
                .collect();

            return Ok(ConsensusResult::Rejected { reasons });
        }

        // Check if all votes are in but no consensus
        if summary.pending == 0 && !summary.has_consensus && !summary.consensus_failed {
            // Check round count
            let current_round =
                ConsensusReview::get_latest_round(&self.pool, swarm_execution_id).await?;

            if current_round >= self.config.max_rounds {
                return Ok(ConsensusResult::Deadlock { summary });
            }
        }

        // Still pending
        Ok(ConsensusResult::Pending { summary })
    }

    /// Finalize consensus and update execution status
    pub async fn finalize_consensus(
        &self,
        swarm_execution_id: Uuid,
    ) -> Result<ConsensusResult, ConsensusError> {
        let result = self.evaluate_consensus(swarm_execution_id).await?;

        match &result {
            ConsensusResult::Approved => {
                SwarmExecution::update_status(
                    &self.pool,
                    swarm_execution_id,
                    SwarmExecutionStatus::Merging,
                )
                .await?;
            }
            ConsensusResult::Rejected { .. } => {
                SwarmExecution::update_status(
                    &self.pool,
                    swarm_execution_id,
                    SwarmExecutionStatus::Failed,
                )
                .await?;
            }
            ConsensusResult::Deadlock { .. } => {
                // Requires human intervention - keep in reviewing state
                // but mark with error message
                SwarmExecution::set_error(
                    &self.pool,
                    swarm_execution_id,
                    "Consensus deadlock - human intervention required",
                )
                .await?;
            }
            ConsensusResult::Pending { .. } => {
                // Do nothing, still waiting for votes
            }
        }

        Ok(result)
    }

    /// Get all reviews for a swarm execution
    pub async fn get_reviews(
        &self,
        swarm_execution_id: Uuid,
    ) -> Result<Vec<ConsensusReview>, ConsensusError> {
        let reviews =
            ConsensusReview::find_by_swarm_execution(&self.pool, swarm_execution_id).await?;
        Ok(reviews)
    }

    /// Get consensus summary
    pub async fn get_summary(
        &self,
        swarm_execution_id: Uuid,
    ) -> Result<ConsensusSummary, ConsensusError> {
        let execution = SwarmExecution::find_by_id(&self.pool, swarm_execution_id)
            .await?
            .ok_or(ConsensusError::ExecutionNotFound(swarm_execution_id))?;

        let summary = ConsensusReview::get_consensus_summary(
            &self.pool,
            swarm_execution_id,
            execution.consensus_threshold,
        )
        .await?;

        Ok(summary)
    }

    /// Generate review prompt for a reviewer agent
    pub fn generate_review_prompt(
        &self,
        execution: &SwarmExecution,
        diff_content: &str,
    ) -> String {
        format!(
            r#"You are reviewing code changes from a swarm execution.

## Instructions
1. Review all code changes carefully
2. Check for:
   - Code quality and best practices
   - Potential bugs or security issues
   - Consistency with the codebase style
   - Test coverage
   - Documentation

3. Provide your vote:
   - APPROVE: Changes are acceptable
   - REJECT: Changes have significant issues
   - ABSTAIN: Unable to make a determination

4. List any issues found with severity (critical/major/minor)
5. Suggest fixes for any issues

## Code Changes
```diff
{diff_content}
```

## Your Review
Please structure your response as:
- **Vote**: [APPROVE/REJECT/ABSTAIN]
- **Confidence**: [0-100]
- **Issues Found**: [List of issues]
- **Suggested Fixes**: [List of fixes]
- **Comments**: [Additional feedback]
"#,
            diff_content = diff_content
        )
    }

    /// Parse review response from agent
    pub fn parse_review_response(&self, response: &str) -> SubmitReviewVote {
        // Simple parsing - in production, use structured output or better parsing
        let vote = if response.to_lowercase().contains("**vote**: approve")
            || response.to_lowercase().contains("vote: approve")
        {
            ConsensusVote::Approve
        } else if response.to_lowercase().contains("**vote**: reject")
            || response.to_lowercase().contains("vote: reject")
        {
            ConsensusVote::Reject
        } else {
            ConsensusVote::Abstain
        };

        // Extract confidence
        let confidence = response
            .lines()
            .find(|l| l.to_lowercase().contains("confidence"))
            .and_then(|l| {
                l.chars()
                    .filter(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse::<i32>()
                    .ok()
            });

        SubmitReviewVote {
            vote,
            comments: Some(response.to_string()),
            structured_feedback: None,
            confidence,
            issues_found: None,
            suggested_fixes: None,
        }
    }
}
