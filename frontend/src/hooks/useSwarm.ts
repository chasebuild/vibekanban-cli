import { useState, useCallback } from 'react';
import { api } from '../lib/api';
import {
  SwarmExecution,
  SwarmTask,
  SwarmProgress,
  SwarmPlanOutput,
  ConsensusReview,
  ConsensusSummary,
  AgentSkill,
  AgentProfile,
  Task,
} from 'shared/types';

interface SwarmExecutionResponse {
  execution: SwarmExecution;
  tasks: SwarmTask[];
  progress: SwarmProgress;
}

interface SwarmPlanResponse {
  execution: SwarmExecution;
  plan: SwarmPlanOutput;
}

interface ConsensusStatusResponse {
  execution: SwarmExecution;
  reviews: ConsensusReview[];
  summary: ConsensusSummary;
}

export function useSwarm() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Swarm Execution operations
  const createSwarmExecution = useCallback(
    async (
      epicTaskId: string,
      workspaceId?: string,
      options?: { reviewerCount?: number; maxParallelWorkers?: number }
    ): Promise<SwarmExecution | null> => {
      setLoading(true);
      setError(null);
      try {
        const response = await api.post<SwarmExecution>('/swarms', {
          epic_task_id: epicTaskId,
          workspace_id: workspaceId,
          reviewer_count: options?.reviewerCount,
          max_parallel_workers: options?.maxParallelWorkers,
        });
        return response.data;
      } catch (err: any) {
        setError(err.message || 'Failed to create swarm execution');
        return null;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  const getSwarmExecution = useCallback(
    async (id: string): Promise<SwarmExecutionResponse | null> => {
      setLoading(true);
      setError(null);
      try {
        const response = await api.get<SwarmExecutionResponse>(`/swarms/${id}`);
        return response.data;
      } catch (err: any) {
        setError(err.message || 'Failed to get swarm execution');
        return null;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  const generatePlan = useCallback(
    async (id: string): Promise<SwarmPlanResponse | null> => {
      setLoading(true);
      setError(null);
      try {
        const response = await api.post<SwarmPlanResponse>(`/swarms/${id}/plan`);
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
    async (id: string): Promise<SwarmTask[] | null> => {
      setLoading(true);
      setError(null);
      try {
        const response = await api.post<SwarmTask[]>(`/swarms/${id}/execute`);
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
    async (id: string): Promise<SwarmProgress | null> => {
      try {
        const response = await api.get<SwarmProgress>(`/swarms/${id}/progress`);
        return response.data;
      } catch (err: any) {
        return null;
      }
    },
    []
  );

  const pauseExecution = useCallback(
    async (id: string): Promise<SwarmExecution | null> => {
      setLoading(true);
      setError(null);
      try {
        const response = await api.post<SwarmExecution>(`/swarms/${id}/pause`);
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
    async (id: string): Promise<SwarmExecution | null> => {
      setLoading(true);
      setError(null);
      try {
        const response = await api.post<SwarmExecution>(`/swarms/${id}/resume`);
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
    async (id: string): Promise<SwarmExecution | null> => {
      setLoading(true);
      setError(null);
      try {
        const response = await api.post<SwarmExecution>(`/swarms/${id}/cancel`);
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

  // Consensus operations
  const getConsensusStatus = useCallback(
    async (id: string): Promise<ConsensusStatusResponse | null> => {
      setLoading(true);
      setError(null);
      try {
        const response = await api.get<ConsensusStatusResponse>(
          `/swarms/${id}/consensus`
        );
        return response.data;
      } catch (err: any) {
        setError(err.message || 'Failed to get consensus status');
        return null;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  const startConsensus = useCallback(
    async (id: string): Promise<ConsensusReview[] | null> => {
      setLoading(true);
      setError(null);
      try {
        const response = await api.post<ConsensusReview[]>(
          `/swarms/${id}/consensus/start`
        );
        return response.data;
      } catch (err: any) {
        setError(err.message || 'Failed to start consensus');
        return null;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  const finalizeConsensus = useCallback(
    async (id: string): Promise<SwarmExecution | null> => {
      setLoading(true);
      setError(null);
      try {
        const response = await api.post<SwarmExecution>(
          `/swarms/${id}/consensus/finalize`
        );
        return response.data;
      } catch (err: any) {
        setError(err.message || 'Failed to finalize consensus');
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
    // Swarm operations
    createSwarmExecution,
    getSwarmExecution,
    generatePlan,
    executePlan,
    getProgress,
    pauseExecution,
    resumeExecution,
    cancelExecution,
    // Consensus operations
    getConsensusStatus,
    startConsensus,
    finalizeConsensus,
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
