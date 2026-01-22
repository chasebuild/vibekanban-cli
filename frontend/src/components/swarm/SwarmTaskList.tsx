import { SwarmTask, SwarmTaskStatus } from '../../../shared/types';
import { Badge } from '../ui/badge';
import {
  CheckCircle,
  Circle,
  Play,
  XCircle,
  Clock,
  SkipForward,
  AlertCircle,
} from 'lucide-react';

interface SwarmTaskListProps {
  tasks: SwarmTask[];
  onTaskClick?: (task: SwarmTask) => void;
}

const statusIcons: Record<SwarmTaskStatus, React.ReactNode> = {
  pending: <Circle className="h-4 w-4 text-gray-400" />,
  blocked: <Clock className="h-4 w-4 text-yellow-500" />,
  assigned: <AlertCircle className="h-4 w-4 text-blue-400" />,
  running: <Play className="h-4 w-4 text-blue-500" />,
  completed: <CheckCircle className="h-4 w-4 text-green-500" />,
  failed: <XCircle className="h-4 w-4 text-red-500" />,
  skipped: <SkipForward className="h-4 w-4 text-gray-500" />,
};

const statusLabels: Record<SwarmTaskStatus, string> = {
  pending: 'Pending',
  blocked: 'Blocked',
  assigned: 'Assigned',
  running: 'Running',
  completed: 'Completed',
  failed: 'Failed',
  skipped: 'Skipped',
};

export function SwarmTaskList({ tasks, onTaskClick }: SwarmTaskListProps) {
  // Sort tasks by sequence order
  const sortedTasks = [...tasks].sort(
    (a, b) => a.sequence_order - b.sequence_order
  );

  return (
    <div className="space-y-2">
      {sortedTasks.map((task, index) => (
        <div
          key={task.id}
          className={`
            border rounded-lg p-3 flex items-center gap-3
            ${onTaskClick ? 'cursor-pointer hover:bg-muted/50' : ''}
            ${task.status === 'running' ? 'border-blue-500 bg-blue-50/50 dark:bg-blue-900/10' : ''}
            ${task.status === 'failed' ? 'border-red-500 bg-red-50/50 dark:bg-red-900/10' : ''}
          `}
          onClick={() => onTaskClick?.(task)}
        >
          {/* Sequence number */}
          <div className="flex-shrink-0 w-6 h-6 rounded-full bg-muted flex items-center justify-center text-xs font-medium">
            {index + 1}
          </div>

          {/* Status icon */}
          <div className="flex-shrink-0">{statusIcons[task.status]}</div>

          {/* Task info */}
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2">
              <span className="font-medium truncate">
                Task {task.id.slice(0, 8)}
              </span>
              <Badge variant="outline" className="text-xs">
                {statusLabels[task.status]}
              </Badge>
            </div>

            {/* Dependencies */}
            {task.depends_on && (
              <div className="text-xs text-muted-foreground mt-1">
                Depends on:{' '}
                {JSON.parse(task.depends_on)
                  .map((id: string) => id.slice(0, 6))
                  .join(', ')}
              </div>
            )}

            {/* Skills */}
            {task.required_skills && (
              <div className="flex gap-1 mt-1">
                {JSON.parse(task.required_skills).map((skill: string) => (
                  <Badge key={skill} variant="secondary" className="text-xs">
                    {skill}
                  </Badge>
                ))}
              </div>
            )}
          </div>

          {/* Complexity */}
          <div className="flex-shrink-0 text-xs text-muted-foreground">
            Complexity: {task.complexity}/5
          </div>

          {/* Duration */}
          {task.duration_seconds && (
            <div className="flex-shrink-0 text-xs text-muted-foreground">
              {Math.round(task.duration_seconds / 60)}m
            </div>
          )}

          {/* Error */}
          {task.error_message && (
            <div className="flex-shrink-0">
              <Badge variant="destructive" className="text-xs">
                Error
              </Badge>
            </div>
          )}
        </div>
      ))}

      {tasks.length === 0 && (
        <div className="text-center py-8 text-muted-foreground">
          No tasks in this swarm execution
        </div>
      )}
    </div>
  );
}
