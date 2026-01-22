import { useState, useEffect, useCallback } from 'react';
import NiceModal, { useModal } from '@ebay/nice-modal-react';
import { defineModal } from '@/lib/modals';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import {
  SwarmExecutionCard,
  SwarmTaskList,
  ConsensusPanel,
} from '@/components/swarm';
import { useSwarm } from '@/hooks/useSwarm';
import {
  Play,
  Pause,
  XCircle,
  RefreshCw,
  CheckCircle,
  GitMerge,
  ListTodo,
  Users,
} from 'lucide-react';
import type {
  SwarmExecution,
  SwarmTask,
  SwarmProgress,
  ConsensusReview,
  ConsensusSummary,
} from 'shared/types';

export interface SwarmExecutionDialogProps {
  taskId: string;
  taskTitle: string;
}

const SwarmExecutionDialogImpl = NiceModal.create<SwarmExecutionDialogProps>(
  ({ taskId, taskTitle }) => {
    const modal = useModal();
    const {
      loading,
      error,
      getSwarmExecution,
      generatePlan,
      executePlan,
      pauseExecution,
      resumeExecution,
      cancelExecution,
      getConsensusStatus,
      startConsensus,
      finalizeConsensus,
    } = useSwarm();

    const [execution, setExecution] = useState<SwarmExecution | null>(null);
    const [tasks, setTasks] = useState<SwarmTask[]>([]);
    const [progress, setProgress] = useState<SwarmProgress | null>(null);
    const [reviews, setReviews] = useState<ConsensusReview[]>([]);
    const [summary, setSummary] = useState<ConsensusSummary | null>(null);
    const [activeTab, setActiveTab] = useState('overview');
    const [swarmId, setSwarmId] = useState<string | null>(null);

    // Find or create swarm execution for this task
    useEffect(() => {
      const fetchSwarmData = async () => {
        // Try to find existing swarm execution for this task
        const response = await fetch(`/api/projects/${taskId}/epic-tasks`);
        if (response.ok) {
          // For now, we'll need the swarm ID passed in or fetched
          // This is a simplified version
        }
      };
      
      if (modal.visible && taskId) {
        fetchSwarmData();
      }
    }, [modal.visible, taskId]);

    const loadSwarmData = useCallback(async () => {
      if (!swarmId) return;

      const swarmData = await getSwarmExecution(swarmId);
      if (swarmData) {
        setExecution(swarmData.execution);
        setTasks(swarmData.tasks);
        setProgress(swarmData.progress);
      }

      // Load consensus data if in review phase
      if (swarmData?.execution.status === 'reviewing') {
        const consensusData = await getConsensusStatus(swarmId);
        if (consensusData) {
          setReviews(consensusData.reviews);
          setSummary(consensusData.summary);
        }
      }
    }, [swarmId, getSwarmExecution, getConsensusStatus]);

    // Poll for updates when active
    useEffect(() => {
      if (!modal.visible || !swarmId) return;

      loadSwarmData();
      const interval = setInterval(loadSwarmData, 5000);
      return () => clearInterval(interval);
    }, [modal.visible, swarmId, loadSwarmData]);

    const handleGeneratePlan = async () => {
      if (!swarmId) return;
      const result = await generatePlan(swarmId);
      if (result) {
        setExecution(result.execution);
      }
    };

    const handleExecutePlan = async () => {
      if (!swarmId) return;
      const newTasks = await executePlan(swarmId);
      if (newTasks) {
        setTasks(newTasks);
        await loadSwarmData();
      }
    };

    const handlePause = async () => {
      if (!swarmId) return;
      const result = await pauseExecution(swarmId);
      if (result) {
        setExecution(result);
      }
    };

    const handleResume = async () => {
      if (!swarmId) return;
      const result = await resumeExecution(swarmId);
      if (result) {
        setExecution(result);
      }
    };

    const handleCancel = async () => {
      if (!swarmId) return;
      const result = await cancelExecution(swarmId);
      if (result) {
        setExecution(result);
      }
    };

    const handleStartReview = async () => {
      if (!swarmId) return;
      const newReviews = await startConsensus(swarmId);
      if (newReviews) {
        setReviews(newReviews);
        await loadSwarmData();
      }
    };

    const handleFinalize = async () => {
      if (!swarmId) return;
      const result = await finalizeConsensus(swarmId);
      if (result) {
        setExecution(result);
      }
    };

    const handleOpenChange = (open: boolean) => {
      if (!open) modal.hide();
    };

    const statusIcon = execution?.status ? (
      {
        planning: <RefreshCw className="h-4 w-4 animate-spin" />,
        planned: <ListTodo className="h-4 w-4" />,
        executing: <Play className="h-4 w-4" />,
        reviewing: <Users className="h-4 w-4" />,
        merging: <GitMerge className="h-4 w-4" />,
        completed: <CheckCircle className="h-4 w-4 text-green-500" />,
        failed: <XCircle className="h-4 w-4 text-red-500" />,
        cancelled: <XCircle className="h-4 w-4 text-gray-500" />,
      }[execution.status]
    ) : null;

    return (
      <Dialog open={modal.visible} onOpenChange={handleOpenChange}>
        <DialogContent className="max-w-4xl max-h-[90vh] overflow-hidden flex flex-col">
          <DialogHeader>
            <div className="flex items-center gap-3">
              {statusIcon}
              <DialogTitle>Agent Swarm: {taskTitle}</DialogTitle>
              {execution && (
                <Badge
                  variant={
                    execution.status === 'completed'
                      ? 'default'
                      : execution.status === 'failed'
                        ? 'destructive'
                        : 'secondary'
                  }
                >
                  {execution.status}
                </Badge>
              )}
            </div>
            <DialogDescription>
              Manage and monitor the agent swarm execution for this epic task.
            </DialogDescription>
          </DialogHeader>

          {!execution ? (
            <div className="flex flex-col items-center justify-center py-12 space-y-4">
              <Users className="h-12 w-12 text-muted-foreground" />
              <p className="text-muted-foreground">
                No swarm execution found for this task.
              </p>
              <Button
                onClick={async () => {
                  // Create a new swarm execution
                  const response = await fetch('/api/swarms', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                      epic_task_id: taskId,
                    }),
                  });
                  if (response.ok) {
                    const newExecution = await response.json();
                    setSwarmId(newExecution.id);
                    setExecution(newExecution);
                  }
                }}
                disabled={loading}
              >
                <Users className="h-4 w-4 mr-2" />
                Create Swarm Execution
              </Button>
            </div>
          ) : (
            <Tabs
              value={activeTab}
              onValueChange={setActiveTab}
              className="flex-1 flex flex-col min-h-0"
            >
              <TabsList className="grid w-full grid-cols-3">
                <TabsTrigger value="overview">Overview</TabsTrigger>
                <TabsTrigger value="tasks">
                  Tasks ({tasks.length})
                </TabsTrigger>
                <TabsTrigger value="consensus">
                  Consensus
                  {execution.status === 'reviewing' && (
                    <span className="ml-1 h-2 w-2 rounded-full bg-yellow-500 animate-pulse" />
                  )}
                </TabsTrigger>
              </TabsList>

              <div className="flex-1 overflow-auto py-4">
                <TabsContent value="overview" className="m-0">
                  {progress && (
                    <SwarmExecutionCard
                      execution={execution}
                      progress={progress}
                      onPause={
                        execution.status === 'executing' ? handlePause : undefined
                      }
                      onResume={
                        execution.status === 'planned' ? handleResume : undefined
                      }
                      onCancel={
                        ['planning', 'planned', 'executing'].includes(
                          execution.status
                        )
                          ? handleCancel
                          : undefined
                      }
                    />
                  )}

                  {/* Action buttons based on state */}
                  <div className="mt-4 flex gap-2">
                    {execution.status === 'planning' && (
                      <Button onClick={handleGeneratePlan} disabled={loading}>
                        <RefreshCw className="h-4 w-4 mr-2" />
                        Generate Plan
                      </Button>
                    )}
                    {execution.status === 'planned' && (
                      <Button onClick={handleExecutePlan} disabled={loading}>
                        <Play className="h-4 w-4 mr-2" />
                        Start Execution
                      </Button>
                    )}
                    {execution.status === 'reviewing' && summary?.has_consensus && (
                      <Button
                        onClick={handleFinalize}
                        disabled={loading}
                        className="bg-green-600"
                      >
                        <GitMerge className="h-4 w-4 mr-2" />
                        Merge Changes
                      </Button>
                    )}
                  </div>
                </TabsContent>

                <TabsContent value="tasks" className="m-0">
                  <SwarmTaskList
                    tasks={tasks}
                    onTaskClick={(task) => {
                      // Could open task details dialog
                      console.log('Task clicked:', task);
                    }}
                  />
                </TabsContent>

                <TabsContent value="consensus" className="m-0">
                  {summary ? (
                    <ConsensusPanel
                      reviews={reviews}
                      summary={summary}
                      onStartReview={
                        reviews.length === 0 ? handleStartReview : undefined
                      }
                      onFinalize={summary.has_consensus ? handleFinalize : undefined}
                    />
                  ) : (
                    <div className="text-center py-8 text-muted-foreground">
                      {execution.status === 'reviewing' ? (
                        <p>Loading consensus data...</p>
                      ) : (
                        <p>
                          Consensus review will be available after all tasks
                          complete.
                        </p>
                      )}
                    </div>
                  )}
                </TabsContent>
              </div>
            </Tabs>
          )}

          {error && (
            <div className="mt-4 p-3 bg-red-50 dark:bg-red-900/20 rounded text-sm text-red-600 dark:text-red-400">
              {error}
            </div>
          )}
        </DialogContent>
      </Dialog>
    );
  }
);

export const SwarmExecutionDialog = defineModal<SwarmExecutionDialogProps, void>(
  SwarmExecutionDialogImpl
);
