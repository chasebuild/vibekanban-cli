import { Task, TaskComplexity } from '../../../shared/types';
import { Badge } from '../ui/badge';
import { Zap, Layers, Crown } from 'lucide-react';

interface EpicTaskBadgeProps {
  task: Task;
  showComplexity?: boolean;
}

const complexityColors: Record<TaskComplexity, string> = {
  trivial: 'bg-gray-500',
  simple: 'bg-blue-500',
  moderate: 'bg-yellow-500',
  complex: 'bg-orange-500',
  epic: 'bg-purple-500',
};

const complexityLabels: Record<TaskComplexity, string> = {
  trivial: 'Trivial',
  simple: 'Simple',
  moderate: 'Moderate',
  complex: 'Complex',
  epic: 'Epic',
};

export function EpicTaskBadge({ task, showComplexity = true }: EpicTaskBadgeProps) {
  if (!task.is_epic) {
    return null;
  }

  return (
    <div className="flex items-center gap-1">
      <Badge className="bg-purple-600 flex items-center gap-1">
        <Crown className="h-3 w-3" />
        Epic
      </Badge>
      {showComplexity && task.complexity && (
        <Badge className={complexityColors[task.complexity]}>
          {complexityLabels[task.complexity]}
        </Badge>
      )}
    </div>
  );
}

interface ComplexityBadgeProps {
  complexity: TaskComplexity | null | undefined;
}

export function ComplexityBadge({ complexity }: ComplexityBadgeProps) {
  if (!complexity) {
    return null;
  }

  return (
    <Badge variant="outline" className={`${complexityColors[complexity]} text-white`}>
      <Layers className="h-3 w-3 mr-1" />
      {complexityLabels[complexity]}
    </Badge>
  );
}
