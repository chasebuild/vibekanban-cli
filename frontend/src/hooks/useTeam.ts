import { useState, useCallback } from 'react';
import { api } from '../lib/api';
import {
  TeamExecution,
  TeamTask,
  TeamProgress,
  TeamPlanOutput,
  AgentSkill,
  AgentProfile,
  Task,
} from 'shared/types';

interface TeamExecutionResponse {
  execution: TeamExecution;
  tasks: TeamTask[];
  progress: TeamProgress;
}

interface TeamPlanResponse {
  execution: TeamExecution;
  plan: TeamPlanOutput;
}

export function useTeam() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Team Execution operations
  const createTeamExecution = useCallback(
    async (
      epicTaskId: string,
      workspaceId?: string,
      options?: { maxParallelWorkers?: number }
    ): Promise<TeamExecution | null> => {
      setLoading(true);
      setError(null);
      try {
        const response = await api.post<TeamExecution>('/teams', {
          epic_task_id: epicTaskId,
          workspace_id: workspaceId,
          max_parallel_workers: options?.maxParallelWorkers,
        });
        return response.data;
      } catch (err: any) {
        setError(err.message || 'Failed to create team execution');
        return null;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  const getTeamExecution = useCallback(
    async (id: string): Promise<TeamExecutionResponse | null> => {
      setLoading(true);
      setError(null);
      try {
        const response = await api.get<TeamExecutionResponse>(`/teams/${id}`);
        return response.data;
      } catch (err: any) {
        setError(err.message || 'Failed to get team execution');
        return null;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  const generatePlan = useCallback(
    async (id: string): Promise<TeamPlanResponse | null> => {
      setLoading(true);
      setError(null);
      try {
        const response = await api.post<TeamPlanResponse>(`/teams/${id}/plan`);
        return response.data;
      } catch (err: any) {
        setError(err.message || 'Failed to generate plan');
        return null;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  const executePlan = useCallback(
    async (id: string): Promise<TeamTask[] | null> => {
      setLoading(true);
      setError(null);
      try {
        const response = await api.post<TeamTask[]>(`/teams/${id}/execute`);
        return response.data;
      } catch (err: any) {
        setError(err.message || 'Failed to execute plan');
        return null;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  const getProgress = useCallback(
    async (id: string): Promise<TeamProgress | null> => {
      try {
        const response = await api.get<TeamProgress>(`/teams/${id}/progress`);
        return response.data;
      } catch (err: any) {
        return null;
      }
    },
    []
  );

  const pauseExecution = useCallback(
    async (id: string): Promise<TeamExecution | null> => {
      setLoading(true);
      setError(null);
      try {
        const response = await api.post<TeamExecution>(`/teams/${id}/pause`);
        return response.data;
      } catch (err: any) {
        setError(err.message || 'Failed to pause execution');
        return null;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  const resumeExecution = useCallback(
    async (id: string): Promise<TeamExecution | null> => {
      setLoading(true);
      setError(null);
      try {
        const response = await api.post<TeamExecution>(`/teams/${id}/resume`);
        return response.data;
      } catch (err: any) {
        setError(err.message || 'Failed to resume execution');
        return null;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  const cancelExecution = useCallback(
    async (id: string): Promise<TeamExecution | null> => {
      setLoading(true);
      setError(null);
      try {
        const response = await api.post<TeamExecution>(`/teams/${id}/cancel`);
        return response.data;
      } catch (err: any) {
        setError(err.message || 'Failed to cancel execution');
        return null;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  // Agent Skills operations
  const listSkills = useCallback(async (): Promise<AgentSkill[] | null> => {
    try {
      const response = await api.get<AgentSkill[]>('/agent-skills');
      return response.data;
    } catch (err: any) {
      return null;
    }
  }, []);

  const createSkill = useCallback(
    async (skill: Partial<AgentSkill>): Promise<AgentSkill | null> => {
      setLoading(true);
      setError(null);
      try {
        const response = await api.post<AgentSkill>('/agent-skills', skill);
        return response.data;
      } catch (err: any) {
        setError(err.message || 'Failed to create skill');
        return null;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  const deleteSkill = useCallback(async (id: string): Promise<boolean> => {
    setLoading(true);
    setError(null);
    try {
      await api.delete(`/agent-skills/${id}`);
      return true;
    } catch (err: any) {
      setError(err.message || 'Failed to delete skill');
      return false;
    } finally {
      setLoading(false);
    }
  }, []);

  // Agent Profiles operations
  const listProfiles = useCallback(async (): Promise<AgentProfile[] | null> => {
    try {
      const response = await api.get<AgentProfile[]>('/agent-profiles');
      return response.data;
    } catch (err: any) {
      return null;
    }
  }, []);

  const createProfile = useCallback(
    async (profile: Partial<AgentProfile>): Promise<AgentProfile | null> => {
      setLoading(true);
      setError(null);
      try {
        const response = await api.post<AgentProfile>('/agent-profiles', profile);
        return response.data;
      } catch (err: any) {
        setError(err.message || 'Failed to create profile');
        return null;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  // Epic Task operations
  const listEpicTasks = useCallback(
    async (projectId: string): Promise<Task[] | null> => {
      try {
        const response = await api.get<Task[]>(
          `/projects/${projectId}/epic-tasks`
        );
        return response.data;
      } catch (err: any) {
        return null;
      }
    },
    []
  );

  const setTaskEpic = useCallback(
    async (taskId: string, isEpic: boolean): Promise<Task | null> => {
      setLoading(true);
      setError(null);
      try {
        const response = await api.post<Task>(`/tasks/${taskId}/set-epic`, {
          is_epic: isEpic,
        });
        return response.data;
      } catch (err: any) {
        setError(err.message || 'Failed to update task');
        return null;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  return {
    loading,
    error,
    // Team operations
    createTeamExecution,
    getTeamExecution,
    generatePlan,
    executePlan,
    getProgress,
    pauseExecution,
    resumeExecution,
    cancelExecution,
    // Skills operations
    listSkills,
    createSkill,
    deleteSkill,
    // Profiles operations
    listProfiles,
    createProfile,
    // Epic task operations
    listEpicTasks,
    setTaskEpic,
  };
}
