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
import { TeamExecutionCard, TeamTaskList } from '@/components/team';
import { useTeam } from '@/hooks/useTeam';
import {
  Play,
  XCircle,
  RefreshCw,
  CheckCircle,
  ListTodo,
  Users,
} from 'lucide-react';
import type { TeamExecution, TeamTask, TeamProgress } from 'shared/types';

export interface TeamExecutionDialogProps {
  taskId: string;
  taskTitle: string;
}

const TeamExecutionDialogImpl = NiceModal.create<TeamExecutionDialogProps>(
  ({ taskId, taskTitle }) => {
    const modal = useModal();
    const {
      loading,
      error,
      getTeamExecution,
      generatePlan,
      executePlan,
      pauseExecution,
      resumeExecution,
      cancelExecution,
    } = useTeam();

    const [execution, setExecution] = useState<TeamExecution | null>(null);
    const [tasks, setTasks] = useState<TeamTask[]>([]);
    const [progress, setProgress] = useState<TeamProgress | null>(null);
    const [activeTab, setActiveTab] = useState('overview');
    const [teamId, setTeamId] = useState<string | null>(null);

    // Find or create team execution for this task
    useEffect(() => {
      const fetchTeamData = async () => {
        // Try to find existing team execution for this task
        const response = await fetch(`/api/projects/${taskId}/epic-tasks`);
        if (response.ok) {
          // For now, we'll need the team ID passed in or fetched
          // This is a simplified version
        }
      };

      if (modal.visible && taskId) {
        fetchTeamData();
      }
    }, [modal.visible, taskId]);

    const loadTeamData = useCallback(async () => {
      if (!teamId) return;

      const teamData = await getTeamExecution(teamId);
      if (teamData) {
        setExecution(teamData.execution);
        setTasks(teamData.tasks);
        setProgress(teamData.progress);
      }
    }, [teamId, getTeamExecution]);

    // Poll for updates when active
    useEffect(() => {
      if (!modal.visible || !teamId) return;

      loadTeamData();
      const interval = setInterval(loadTeamData, 5000);
      return () => clearInterval(interval);
    }, [modal.visible, teamId, loadTeamData]);

    const handleGeneratePlan = async () => {
      if (!teamId) return;
      const result = await generatePlan(teamId);
      if (result) {
        setExecution(result.execution);
      }
    };

    const handleExecutePlan = async () => {
      if (!teamId) return;
      const newTasks = await executePlan(teamId);
      if (newTasks) {
        setTasks(newTasks);
        await loadTeamData();
      }
    };

    const handlePause = async () => {
      if (!teamId) return;
      const result = await pauseExecution(teamId);
      if (result) {
        setExecution(result);
      }
    };

    const handleResume = async () => {
      if (!teamId) return;
      const result = await resumeExecution(teamId);
      if (result) {
        setExecution(result);
      }
    };

    const handleCancel = async () => {
      if (!teamId) return;
      const result = await cancelExecution(teamId);
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
              <DialogTitle>Agent Team: {taskTitle}</DialogTitle>
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
              Manage and monitor the agent team execution for this epic task.
            </DialogDescription>
          </DialogHeader>

          {!execution ? (
            <div className="flex flex-col items-center justify-center py-12 space-y-4">
              <Users className="h-12 w-12 text-muted-foreground" />
              <p className="text-muted-foreground">
                No team execution found for this task.
              </p>
              <Button
                onClick={async () => {
                  // Create a new team execution
                  const response = await fetch('/api/teams', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                      epic_task_id: taskId,
                    }),
                  });
                  if (response.ok) {
                    const newExecution = await response.json();
                    setTeamId(newExecution.id);
                    setExecution(newExecution);
                  }
                }}
                disabled={loading}
              >
                <Users className="h-4 w-4 mr-2" />
                Create Team Execution
              </Button>
            </div>
          ) : (
            <Tabs
              value={activeTab}
              onValueChange={setActiveTab}
              className="flex-1 flex flex-col min-h-0"
            >
              <TabsList className="grid w-full grid-cols-2">
                <TabsTrigger value="overview">Overview</TabsTrigger>
                <TabsTrigger value="tasks">Tasks ({tasks.length})</TabsTrigger>
              </TabsList>

              <div className="flex-1 overflow-auto py-4">
                <TabsContent value="overview" className="m-0">
                  {progress && (
                    <TeamExecutionCard
                      execution={execution}
                      progress={progress}
                      onPause={
                        execution.status === 'executing'
                          ? handlePause
                          : undefined
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
                  </div>
                </TabsContent>

                <TabsContent value="tasks" className="m-0">
                  <TeamTaskList
                    tasks={tasks}
                    onTaskClick={(task) => {
                      // Could open task details dialog
                      console.log('Task clicked:', task);
                    }}
                  />
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

export const TeamExecutionDialog = defineModal<TeamExecutionDialogProps, void>(
  TeamExecutionDialogImpl
);
