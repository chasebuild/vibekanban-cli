import { useMemo } from 'react';
import {
  TeamExecution,
  TeamProgress,
  TeamExecutionStatus,
} from '../../../shared/types';
import { Progress } from '../ui/progress';
import { Badge } from '../ui/badge';
import { Button } from '../ui/button';
import {
  Play,
  Pause,
  XCircle,
  CheckCircle,
  Clock,
  Users,
  GitBranch,
} from 'lucide-react';

interface TeamExecutionCardProps {
  execution: TeamExecution;
  progress: TeamProgress;
  onPause?: () => void;
  onResume?: () => void;
  onCancel?: () => void;
  onViewDetails?: () => void;
}

const statusColors: Record<TeamExecutionStatus, string> = {
  planning: 'bg-yellow-500',
  planned: 'bg-blue-500',
  executing: 'bg-green-500',
  completed: 'bg-emerald-500',
  failed: 'bg-red-500',
  cancelled: 'bg-gray-500',
};

const statusLabels: Record<TeamExecutionStatus, string> = {
  planning: 'Planning',
  planned: 'Ready',
  executing: 'Executing',
  completed: 'Completed',
  failed: 'Failed',
  cancelled: 'Cancelled',
};

export function TeamExecutionCard({
  execution,
  progress,
  onPause,
  onResume,
  onCancel,
  onViewDetails,
}: TeamExecutionCardProps) {
  const progressPercentage = useMemo(() => {
    if (progress.total === 0) return 0;
    return Math.round((progress.completed / progress.total) * 100);
  }, [progress]);

  const isActive = execution.status === 'executing';
  const isPaused = execution.status === 'planned';
  const canControl =
    execution.status !== 'completed' &&
    execution.status !== 'failed' &&
    execution.status !== 'cancelled';

  return (
    <div className="border rounded-lg p-4 bg-card">
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <Badge className={statusColors[execution.status]}>
            {statusLabels[execution.status]}
          </Badge>
          <span className="text-sm text-muted-foreground">
            {execution.id.slice(0, 8)}
          </span>
        </div>
        <div className="flex items-center gap-2">
          {canControl && (
            <>
              {isActive && onPause && (
                <Button variant="outline" size="sm" onClick={onPause}>
                  <Pause className="h-4 w-4 mr-1" />
                  Pause
                </Button>
              )}
              {isPaused && onResume && (
                <Button variant="outline" size="sm" onClick={onResume}>
                  <Play className="h-4 w-4 mr-1" />
                  Resume
                </Button>
              )}
              {onCancel && (
                <Button variant="destructive" size="sm" onClick={onCancel}>
                  <XCircle className="h-4 w-4 mr-1" />
                  Cancel
                </Button>
              )}
            </>
          )}
        </div>
      </div>

      {/* Progress */}
      <div className="space-y-2 mb-4">
        <div className="flex items-center justify-between text-sm">
          <span>Progress</span>
          <span className="font-medium">{progressPercentage}%</span>
        </div>
        <Progress value={progressPercentage} />
      </div>

      {/* Stats */}
      <div className="grid grid-cols-4 gap-4 text-sm">
        <div className="flex flex-col items-center p-2 bg-muted rounded">
          <CheckCircle className="h-4 w-4 text-green-500 mb-1" />
          <span className="font-medium">{progress.completed}</span>
          <span className="text-xs text-muted-foreground">Completed</span>
        </div>
        <div className="flex flex-col items-center p-2 bg-muted rounded">
          <Play className="h-4 w-4 text-blue-500 mb-1" />
          <span className="font-medium">{progress.running}</span>
          <span className="text-xs text-muted-foreground">Running</span>
        </div>
        <div className="flex flex-col items-center p-2 bg-muted rounded">
          <Clock className="h-4 w-4 text-yellow-500 mb-1" />
          <span className="font-medium">{progress.pending}</span>
          <span className="text-xs text-muted-foreground">Pending</span>
        </div>
        <div className="flex flex-col items-center p-2 bg-muted rounded">
          <XCircle className="h-4 w-4 text-red-500 mb-1" />
          <span className="font-medium">{progress.failed}</span>
          <span className="text-xs text-muted-foreground">Failed</span>
        </div>
      </div>

      {/* Metadata */}
      <div className="mt-4 pt-4 border-t flex items-center justify-between text-sm text-muted-foreground">
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-1">
            <Users className="h-4 w-4" />
            <span>{execution.max_parallel_workers} workers</span>
          </div>
          <div className="flex items-center gap-1">
            <GitBranch className="h-4 w-4" />
            <span>Team branches</span>
          </div>
        </div>
        {onViewDetails && (
          <Button variant="ghost" size="sm" onClick={onViewDetails}>
            View Details
          </Button>
        )}
      </div>

      {/* Error message */}
      {execution.error_message && (
        <div className="mt-4 p-3 bg-red-50 dark:bg-red-900/20 rounded text-sm text-red-600 dark:text-red-400">
          {execution.error_message}
        </div>
      )}
    </div>
  );
}
