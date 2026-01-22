import {
  ConsensusReview,
  ConsensusSummary,
  ConsensusVote,
} from '../../../shared/types';
import { Badge } from '../ui/badge';
import { Progress } from '../ui/progress';
import { Button } from '../ui/button';
import {
  CheckCircle,
  XCircle,
  HelpCircle,
  Clock,
  Users,
  ThumbsUp,
  ThumbsDown,
} from 'lucide-react';

interface ConsensusPanelProps {
  reviews: ConsensusReview[];
  summary: ConsensusSummary;
  onStartReview?: () => void;
  onFinalize?: () => void;
}

const voteIcons: Record<ConsensusVote, React.ReactNode> = {
  pending: <Clock className="h-4 w-4 text-gray-400" />,
  approve: <ThumbsUp className="h-4 w-4 text-green-500" />,
  reject: <ThumbsDown className="h-4 w-4 text-red-500" />,
  abstain: <HelpCircle className="h-4 w-4 text-yellow-500" />,
};

const voteLabels: Record<ConsensusVote, string> = {
  pending: 'Pending',
  approve: 'Approve',
  reject: 'Reject',
  abstain: 'Abstain',
};

export function ConsensusPanel({
  reviews,
  summary,
  onStartReview,
  onFinalize,
}: ConsensusPanelProps) {
  const approvalProgress =
    summary.total_reviewers > 0
      ? Math.round((summary.approvals / summary.threshold) * 100)
      : 0;

  return (
    <div className="border rounded-lg p-4 bg-card">
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <Users className="h-5 w-5" />
          <h3 className="font-semibold">Consensus Review</h3>
        </div>
        <div className="flex items-center gap-2">
          {summary.has_consensus && (
            <Badge className="bg-green-500">Consensus Reached</Badge>
          )}
          {summary.consensus_failed && (
            <Badge className="bg-red-500">Consensus Failed</Badge>
          )}
          {!summary.has_consensus && !summary.consensus_failed && (
            <Badge variant="outline">In Progress</Badge>
          )}
        </div>
      </div>

      {/* Progress to consensus */}
      <div className="mb-4">
        <div className="flex items-center justify-between text-sm mb-2">
          <span>Approval Progress</span>
          <span className="font-medium">
            {summary.approvals} / {summary.threshold} required
          </span>
        </div>
        <Progress
          value={Math.min(approvalProgress, 100)}
          className={summary.has_consensus ? 'bg-green-100' : ''}
        />
      </div>

      {/* Vote summary */}
      <div className="grid grid-cols-4 gap-3 mb-4">
        <div className="flex flex-col items-center p-2 bg-green-50 dark:bg-green-900/20 rounded">
          <ThumbsUp className="h-4 w-4 text-green-500 mb-1" />
          <span className="font-medium text-green-600">{summary.approvals}</span>
          <span className="text-xs text-muted-foreground">Approve</span>
        </div>
        <div className="flex flex-col items-center p-2 bg-red-50 dark:bg-red-900/20 rounded">
          <ThumbsDown className="h-4 w-4 text-red-500 mb-1" />
          <span className="font-medium text-red-600">{summary.rejections}</span>
          <span className="text-xs text-muted-foreground">Reject</span>
        </div>
        <div className="flex flex-col items-center p-2 bg-yellow-50 dark:bg-yellow-900/20 rounded">
          <HelpCircle className="h-4 w-4 text-yellow-500 mb-1" />
          <span className="font-medium text-yellow-600">
            {summary.abstentions}
          </span>
          <span className="text-xs text-muted-foreground">Abstain</span>
        </div>
        <div className="flex flex-col items-center p-2 bg-gray-50 dark:bg-gray-800 rounded">
          <Clock className="h-4 w-4 text-gray-400 mb-1" />
          <span className="font-medium">{summary.pending}</span>
          <span className="text-xs text-muted-foreground">Pending</span>
        </div>
      </div>

      {/* Individual reviews */}
      <div className="space-y-2 mb-4">
        <h4 className="text-sm font-medium">Reviewer Votes</h4>
        {reviews.map((review) => (
          <div
            key={review.id}
            className="flex items-center justify-between p-2 bg-muted rounded"
          >
            <div className="flex items-center gap-2">
              {voteIcons[review.vote]}
              <span className="text-sm">
                Reviewer {review.reviewer_profile_id.slice(0, 8)}
              </span>
            </div>
            <div className="flex items-center gap-2">
              {review.confidence !== null && (
                <span className="text-xs text-muted-foreground">
                  {review.confidence}% confidence
                </span>
              )}
              <Badge
                variant={review.vote === 'approve' ? 'default' : 'outline'}
              >
                {voteLabels[review.vote]}
              </Badge>
            </div>
          </div>
        ))}

        {reviews.length === 0 && (
          <div className="text-center py-4 text-muted-foreground text-sm">
            No reviews started yet
          </div>
        )}
      </div>

      {/* Actions */}
      <div className="flex gap-2">
        {reviews.length === 0 && onStartReview && (
          <Button onClick={onStartReview} className="flex-1">
            Start Review Process
          </Button>
        )}
        {summary.has_consensus && onFinalize && (
          <Button onClick={onFinalize} className="flex-1 bg-green-600">
            <CheckCircle className="h-4 w-4 mr-2" />
            Finalize & Merge
          </Button>
        )}
        {summary.consensus_failed && (
          <div className="flex-1 p-3 bg-red-50 dark:bg-red-900/20 rounded text-sm text-red-600">
            Consensus failed. Review the feedback and consider making changes.
          </div>
        )}
      </div>

      {/* pBFT explanation */}
      <div className="mt-4 pt-4 border-t">
        <p className="text-xs text-muted-foreground">
          Using pBFT consensus: requires {summary.threshold} approvals from{' '}
          {summary.total_reviewers} reviewers (tolerates up to{' '}
          {Math.floor((summary.total_reviewers - 1) / 3)} faulty reviewers)
        </p>
      </div>
    </div>
  );
}
