import { useQuery } from '@tanstack/react-query';
import { tasksApi } from '@/lib/api';
import type { ProjectTaskStats } from 'shared/types';

export interface UseAllProjectsTaskStatsResult {
  stats: ProjectTaskStats[];
  statsByProjectId: Record<string, ProjectTaskStats>;
  isLoading: boolean;
  error: Error | null;
}

/**
 * Fetch task stats for all projects.
 * Returns stats array and a map keyed by project_id for easy lookup.
 */
export const useAllProjectsTaskStats = (): UseAllProjectsTaskStatsResult => {
  const {
    data: stats = [],
    isLoading,
    error,
  } = useQuery({
    queryKey: ['all-projects-task-stats'],
    queryFn: () => tasksApi.getAllProjectsStats(),
    staleTime: 10000, // 10 seconds - refresh relatively frequently for live-ish data
    refetchInterval: 30000, // Refetch every 30 seconds for running tasks
  });

  const statsByProjectId: Record<string, ProjectTaskStats> = {};
  for (const stat of stats) {
    statsByProjectId[stat.project_id] = stat;
  }

  return {
    stats,
    statsByProjectId,
    isLoading,
    error: error as Error | null,
  };
};
