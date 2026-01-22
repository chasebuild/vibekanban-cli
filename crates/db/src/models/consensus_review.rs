use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool, Type};
use strum_macros::{Display, EnumString};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, Type, Serialize, Deserialize, PartialEq, TS, EnumString, Display, Default)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ConsensusVote {
    #[default]
    Pending,
    Approve,
    Reject,
    Abstain,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, TS)]
pub struct ConsensusReview {
    pub id: Uuid,
    pub swarm_execution_id: Uuid,
    pub reviewer_profile_id: Uuid,
    pub session_id: Option<Uuid>,
    pub vote: ConsensusVote,
    pub comments: Option<String>,
    pub structured_feedback: Option<String>,
    pub review_diff_hash: Option<String>,
    pub confidence: Option<i32>,
    pub issues_found: Option<String>,
    pub suggested_fixes: Option<String>,
    pub round: i32,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct CreateConsensusReview {
    pub swarm_execution_id: Uuid,
    pub reviewer_profile_id: Uuid,
    pub round: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct SubmitReviewVote {
    pub vote: ConsensusVote,
    pub comments: Option<String>,
    pub structured_feedback: Option<String>,
    pub confidence: Option<i32>,
    pub issues_found: Option<Vec<String>>,
    pub suggested_fixes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct ConsensusReviewWithProfile {
    #[serde(flatten)]
    #[ts(flatten)]
    pub review: ConsensusReview,
    pub reviewer_name: String,
    pub reviewer_executor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct ConsensusSummary {
    pub total_reviewers: i32,
    pub votes_cast: i32,
    pub approvals: i32,
    pub rejections: i32,
    pub abstentions: i32,
    pub pending: i32,
    pub threshold: i32,
    pub has_consensus: bool,
    pub consensus_failed: bool,
}

impl ConsensusReview {
    pub async fn find_by_id(pool: &SqlitePool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            ConsensusReview,
            r#"SELECT 
                id AS "id!: Uuid",
                swarm_execution_id AS "swarm_execution_id!: Uuid",
                reviewer_profile_id AS "reviewer_profile_id!: Uuid",
                session_id AS "session_id: Uuid",
                vote AS "vote!: ConsensusVote",
                comments,
                structured_feedback,
                review_diff_hash,
                confidence AS "confidence: i32",
                issues_found,
                suggested_fixes,
                round AS "round!: i32",
                started_at AS "started_at: DateTime<Utc>",
                completed_at AS "completed_at: DateTime<Utc>",
                created_at AS "created_at!: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>"
            FROM consensus_reviews
            WHERE id = $1"#,
            id
        )
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_swarm_execution(
        pool: &SqlitePool,
        swarm_execution_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            ConsensusReview,
            r#"SELECT 
                id AS "id!: Uuid",
                swarm_execution_id AS "swarm_execution_id!: Uuid",
                reviewer_profile_id AS "reviewer_profile_id!: Uuid",
                session_id AS "session_id: Uuid",
                vote AS "vote!: ConsensusVote",
                comments,
                structured_feedback,
                review_diff_hash,
                confidence AS "confidence: i32",
                issues_found,
                suggested_fixes,
                round AS "round!: i32",
                started_at AS "started_at: DateTime<Utc>",
                completed_at AS "completed_at: DateTime<Utc>",
                created_at AS "created_at!: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>"
            FROM consensus_reviews
            WHERE swarm_execution_id = $1
            ORDER BY round, created_at"#,
            swarm_execution_id
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_round(
        pool: &SqlitePool,
        swarm_execution_id: Uuid,
        round: i32,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            ConsensusReview,
            r#"SELECT 
                id AS "id!: Uuid",
                swarm_execution_id AS "swarm_execution_id!: Uuid",
                reviewer_profile_id AS "reviewer_profile_id!: Uuid",
                session_id AS "session_id: Uuid",
                vote AS "vote!: ConsensusVote",
                comments,
                structured_feedback,
                review_diff_hash,
                confidence AS "confidence: i32",
                issues_found,
                suggested_fixes,
                round AS "round!: i32",
                started_at AS "started_at: DateTime<Utc>",
                completed_at AS "completed_at: DateTime<Utc>",
                created_at AS "created_at!: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>"
            FROM consensus_reviews
            WHERE swarm_execution_id = $1 AND round = $2
            ORDER BY created_at"#,
            swarm_execution_id,
            round
        )
        .fetch_all(pool)
        .await
    }

    pub async fn create(pool: &SqlitePool, data: &CreateConsensusReview) -> Result<Self, sqlx::Error> {
        let id = Uuid::new_v4();
        let round = data.round.unwrap_or(1);

        sqlx::query_as!(
            ConsensusReview,
            r#"INSERT INTO consensus_reviews 
                (id, swarm_execution_id, reviewer_profile_id, round)
            VALUES ($1, $2, $3, $4)
            RETURNING 
                id AS "id!: Uuid",
                swarm_execution_id AS "swarm_execution_id!: Uuid",
                reviewer_profile_id AS "reviewer_profile_id!: Uuid",
                session_id AS "session_id: Uuid",
                vote AS "vote!: ConsensusVote",
                comments,
                structured_feedback,
                review_diff_hash,
                confidence AS "confidence: i32",
                issues_found,
                suggested_fixes,
                round AS "round!: i32",
                started_at AS "started_at: DateTime<Utc>",
                completed_at AS "completed_at: DateTime<Utc>",
                created_at AS "created_at!: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>""#,
            id,
            data.swarm_execution_id,
            data.reviewer_profile_id,
            round
        )
        .fetch_one(pool)
        .await
    }

    pub async fn start_review(
        pool: &SqlitePool,
        id: Uuid,
        session_id: Uuid,
        diff_hash: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE consensus_reviews SET session_id = $2, review_diff_hash = $3, started_at = datetime('now', 'subsec'), updated_at = datetime('now', 'subsec') WHERE id = $1",
            id, session_id, diff_hash
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn submit_vote(
        pool: &SqlitePool,
        id: Uuid,
        data: &SubmitReviewVote,
    ) -> Result<Self, sqlx::Error> {
        let issues = data.issues_found.as_ref().map(|i| serde_json::to_string(i).unwrap_or_default());
        let fixes = data.suggested_fixes.as_ref().map(|f| serde_json::to_string(f).unwrap_or_default());

        sqlx::query_as!(
            ConsensusReview,
            r#"UPDATE consensus_reviews SET 
                vote = $2,
                comments = $3,
                structured_feedback = $4,
                confidence = $5,
                issues_found = $6,
                suggested_fixes = $7,
                completed_at = datetime('now', 'subsec'),
                updated_at = datetime('now', 'subsec')
            WHERE id = $1
            RETURNING 
                id AS "id!: Uuid",
                swarm_execution_id AS "swarm_execution_id!: Uuid",
                reviewer_profile_id AS "reviewer_profile_id!: Uuid",
                session_id AS "session_id: Uuid",
                vote AS "vote!: ConsensusVote",
                comments,
                structured_feedback,
                review_diff_hash,
                confidence AS "confidence: i32",
                issues_found,
                suggested_fixes,
                round AS "round!: i32",
                started_at AS "started_at: DateTime<Utc>",
                completed_at AS "completed_at: DateTime<Utc>",
                created_at AS "created_at!: DateTime<Utc>",
                updated_at AS "updated_at!: DateTime<Utc>""#,
            id,
            data.vote,
            data.comments,
            data.structured_feedback,
            data.confidence,
            issues,
            fixes
        )
        .fetch_one(pool)
        .await
    }

    /// Get consensus summary for a swarm execution
    pub async fn get_consensus_summary(
        pool: &SqlitePool,
        swarm_execution_id: Uuid,
        threshold: i32,
    ) -> Result<ConsensusSummary, sqlx::Error> {
        let result = sqlx::query!(
            r#"SELECT 
                COUNT(*) AS "total!: i64",
                SUM(CASE WHEN vote != 'pending' THEN 1 ELSE 0 END) AS "votes_cast!: i64",
                SUM(CASE WHEN vote = 'approve' THEN 1 ELSE 0 END) AS "approvals!: i64",
                SUM(CASE WHEN vote = 'reject' THEN 1 ELSE 0 END) AS "rejections!: i64",
                SUM(CASE WHEN vote = 'abstain' THEN 1 ELSE 0 END) AS "abstentions!: i64",
                SUM(CASE WHEN vote = 'pending' THEN 1 ELSE 0 END) AS "pending!: i64"
            FROM consensus_reviews
            WHERE swarm_execution_id = $1"#,
            swarm_execution_id
        )
        .fetch_one(pool)
        .await?;

        let total = result.total as i32;
        let approvals = result.approvals as i32;
        let rejections = result.rejections as i32;

        Ok(ConsensusSummary {
            total_reviewers: total,
            votes_cast: result.votes_cast as i32,
            approvals,
            rejections,
            abstentions: result.abstentions as i32,
            pending: result.pending as i32,
            threshold,
            has_consensus: approvals >= threshold,
            consensus_failed: rejections > total - threshold,
        })
    }

    /// Get the latest round number for a swarm execution
    pub async fn get_latest_round(
        pool: &SqlitePool,
        swarm_execution_id: Uuid,
    ) -> Result<i32, sqlx::Error> {
        let result = sqlx::query!(
            r#"SELECT COALESCE(MAX(round), 0) AS "round!: i32" FROM consensus_reviews WHERE swarm_execution_id = $1"#,
            swarm_execution_id
        )
        .fetch_one(pool)
        .await?;
        Ok(result.round)
    }

    pub fn get_issues(&self) -> Vec<String> {
        self.issues_found
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default()
    }

    pub fn get_suggested_fixes(&self) -> Vec<String> {
        self.suggested_fixes
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default()
    }
}
