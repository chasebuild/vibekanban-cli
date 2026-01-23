import { useState, useCallback, useMemo, useRef, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { useDropzone } from 'react-dropzone';
import {
  Send,
  Loader2,
  Plus,
  X,
  GitBranch,
  ChevronDown,
  ChevronUp,
  Check,
  Search,
  Image as ImageIcon,
  Paperclip,
  MessageSquarePlus,
  User,
  Users,
  Crown,
  Zap,
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { AutoExpandingTextarea } from '@/components/ui/auto-expanding-textarea';
import { Switch } from '@/components/ui/switch';
import { Label } from '@/components/ui/label';
import { Slider } from '@/components/ui/slider';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { ExecutorProfileSelector } from '@/components/settings';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import {
  useTaskMutations,
  useProjectRepos,
  useRepoBranchSelection,
  useImageUpload,
} from '@/hooks';
import { useUserSystem } from '@/components/ConfigProvider';
import { cn } from '@/lib/utils';
import type {
  ExecutorProfileId,
  GitBranch as GitBranchType,
  ImageResponse,
} from 'shared/types';

// Execution mode type
type ExecutionMode = 'single' | 'swarm';

interface TaskRow {
  id: string;
  prompt: string;
  branch: string | null;
  images: ImageResponse[];
}

interface QuickTaskInputProps {
  projectId: string;
  className?: string;
  /** Start collapsed (default: false) */
  defaultCollapsed?: boolean;
}

// Branch selector component with support for custom branch names
interface BranchComboboxProps {
  branches: GitBranchType[];
  value: string | null;
  onChange: (value: string) => void;
  placeholder?: string;
  disabled?: boolean;
  isLoading?: boolean;
}

function BranchCombobox({
  branches,
  value,
  onChange,
  placeholder = 'Select branch...',
  disabled,
  isLoading,
}: BranchComboboxProps) {
  const { t } = useTranslation(['common']);
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState('');
  const inputRef = useRef<HTMLInputElement>(null);

  const filteredBranches = useMemo(() => {
    if (!search.trim()) return branches;
    const q = search.toLowerCase();
    return branches.filter((b) => b.name.toLowerCase().includes(q));
  }, [branches, search]);

  // Check if search matches any existing branch - allow creating new branches
  const exactMatch = branches.find(
    (b) => b.name.toLowerCase() === search.trim().toLowerCase()
  );
  const showCreateOption = search.trim() && !exactMatch;

  useEffect(() => {
    if (open && inputRef.current) {
      // Small delay to ensure the dropdown is rendered
      setTimeout(() => inputRef.current?.focus(), 50);
    }
    if (!open) {
      setSearch('');
    }
  }, [open]);

  const handleSelect = useCallback(
    (branchName: string) => {
      onChange(branchName);
      setOpen(false);
      setSearch('');
    },
    [onChange]
  );

  return (
    <DropdownMenu open={open} onOpenChange={setOpen}>
      <DropdownMenuTrigger asChild>
        <Button
          variant="outline"
          size="sm"
          disabled={disabled || isLoading}
          className="w-full justify-between text-xs h-8"
        >
          <div className="flex items-center gap-1.5 min-w-0 flex-1">
            <GitBranch className="h-3 w-3 flex-shrink-0" />
            <span className="truncate">
              {isLoading ? t('common:loading') : value || placeholder}
            </span>
          </div>
          <ChevronDown className="h-3 w-3 flex-shrink-0 opacity-50" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent className="w-56" align="start">
        <div className="p-2">
          <div className="relative">
            <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
            <Input
              ref={inputRef}
              placeholder={t('branchSelector.searchPlaceholder')}
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              onKeyDown={(e) => {
                // Create custom branch on Enter if search doesn't match any branch
                if (e.key === 'Enter') {
                  e.preventDefault();
                  if (showCreateOption) {
                    handleSelect(search.trim());
                  } else if (filteredBranches.length > 0) {
                    handleSelect(filteredBranches[0].name);
                  }
                }
                e.stopPropagation();
              }}
              className="pl-8 h-8 text-xs"
            />
          </div>
        </div>
        <DropdownMenuSeparator />
        <div className="max-h-48 overflow-y-auto">
          {showCreateOption && (
            <DropdownMenuItem
              onSelect={() => handleSelect(search.trim())}
              className="text-xs"
            >
              <Plus className="h-3 w-3 mr-2" />
              {t('branchSelector.useCustom', { branch: search.trim() })}
            </DropdownMenuItem>
          )}
          {filteredBranches.length === 0 && !showCreateOption ? (
            <div className="p-2 text-xs text-center text-muted-foreground">
              {t('branchSelector.empty')}
            </div>
          ) : (
            filteredBranches.map((branch) => (
              <DropdownMenuItem
                key={branch.name}
                onSelect={() => handleSelect(branch.name)}
                className="text-xs"
              >
                <Check
                  className={cn(
                    'mr-2 h-3 w-3',
                    value === branch.name ? 'opacity-100' : 'opacity-0'
                  )}
                />
                <span className="truncate flex-1">{branch.name}</span>
                {branch.is_current && (
                  <span className="text-[10px] bg-muted px-1 rounded ml-1">
                    {t('branchSelector.badges.current')}
                  </span>
                )}
              </DropdownMenuItem>
            ))
          )}
        </div>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

// Image thumbnail component
function ImageThumbnail({
  image,
  onRemove,
  disabled,
}: {
  image: ImageResponse;
  onRemove: () => void;
  disabled: boolean;
}) {
  return (
    <div className="relative group w-16 h-16 rounded-md overflow-hidden border bg-muted flex-shrink-0">
      <img
        src={image.file_path}
        alt={image.original_name}
        className="w-full h-full object-cover"
      />
      {!disabled && (
        <button
          onClick={onRemove}
          className="absolute top-0.5 right-0.5 p-0.5 rounded-full bg-black/60 text-white opacity-0 group-hover:opacity-100 transition-opacity"
        >
          <X className="h-3 w-3" />
        </button>
      )}
    </div>
  );
}

// Single task row component with multiline textarea and image support
interface TaskRowInputProps {
  row: TaskRow;
  branches: GitBranchType[];
  branchesLoading: boolean;
  onChange: (
    id: string,
    field: 'prompt' | 'branch',
    value: string
  ) => void;
  onImagesChange: (id: string, images: ImageResponse[]) => void;
  onRemove: (id: string) => void;
  onKeyDown: (e: React.KeyboardEvent, id: string) => void;
  showRemove: boolean;
  disabled: boolean;
  autoFocus?: boolean;
  placeholder: string;
  upload: (file: File) => Promise<ImageResponse>;
}

function TaskRowInput({
  row,
  branches,
  branchesLoading,
  onChange,
  onImagesChange,
  onRemove,
  onKeyDown,
  showRemove,
  disabled,
  autoFocus,
  placeholder,
  upload,
}: TaskRowInputProps) {
  const { t } = useTranslation(['tasks']);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (autoFocus && textareaRef.current) {
      textareaRef.current.focus();
    }
  }, [autoFocus]);

  const handleDrop = useCallback(
    async (acceptedFiles: File[]) => {
      for (const file of acceptedFiles) {
        try {
          const img = await upload(file);
          onImagesChange(row.id, [...row.images, img]);
        } catch {
          // Silently ignore upload errors
        }
      }
    },
    [upload, row.id, row.images, onImagesChange]
  );

  const { getRootProps, getInputProps, isDragActive, open: openDropzone } =
    useDropzone({
      onDrop: handleDrop,
      accept: { 'image/*': [] },
      disabled: disabled,
      noClick: true,
      noKeyboard: true,
    });

  const handlePaste = useCallback(
    async (e: React.ClipboardEvent) => {
      const items = e.clipboardData.items;
      const imageFiles: File[] = [];

      for (const item of items) {
        if (item.type.startsWith('image/')) {
          const file = item.getAsFile();
          if (file) {
            imageFiles.push(file);
          }
        }
      }

      if (imageFiles.length > 0) {
        e.preventDefault();
        for (const file of imageFiles) {
          try {
            const img = await upload(file);
            onImagesChange(row.id, [...row.images, img]);
          } catch {
            // Silently ignore upload errors
          }
        }
      }
    },
    [upload, row.id, row.images, onImagesChange]
  );

  const handleRemoveImage = useCallback(
    (imageId: string) => {
      onImagesChange(
        row.id,
        row.images.filter((img) => img.id !== imageId)
      );
    },
    [row.id, row.images, onImagesChange]
  );

  const handleAttachClick = () => {
    openDropzone();
  };

  return (
    <div
      {...getRootProps()}
      className={cn(
        'relative rounded-lg border bg-background transition-colors',
        isDragActive && 'border-primary bg-primary/5',
        'group'
      )}
    >
      <input {...getInputProps()} />
      <input
        ref={fileInputRef}
        type="file"
        accept="image/*"
        multiple
        className="hidden"
        onChange={(e) => {
          const files = Array.from(e.target.files || []);
          handleDrop(files);
          e.target.value = '';
        }}
      />

      {/* Drag overlay */}
      {isDragActive && (
        <div className="absolute inset-0 z-10 rounded-lg bg-primary/10 border-2 border-dashed border-primary flex items-center justify-center pointer-events-none">
          <div className="text-center">
            <ImageIcon className="h-8 w-8 mx-auto mb-2 text-primary" />
            <p className="text-sm font-medium">{t('dropzone.dropImagesHere')}</p>
          </div>
        </div>
      )}

      {/* Main content */}
      <div className="p-3 space-y-2">
        {/* Textarea */}
        <AutoExpandingTextarea
          ref={textareaRef}
          value={row.prompt}
          onChange={(e) => onChange(row.id, 'prompt', e.target.value)}
          onKeyDown={(e) => onKeyDown(e, row.id)}
          onPaste={handlePaste}
          placeholder={placeholder}
          disabled={disabled}
          maxRows={8}
          className="w-full bg-transparent text-sm resize-none border-0 focus:ring-0 p-0 min-h-[60px] placeholder:text-muted-foreground"
        />

        {/* Image previews */}
        {row.images.length > 0 && (
          <div className="flex flex-wrap gap-2 pt-1">
            {row.images.map((image) => (
              <ImageThumbnail
                key={image.id}
                image={image}
                onRemove={() => handleRemoveImage(image.id)}
                disabled={disabled}
              />
            ))}
          </div>
        )}

        {/* Row footer */}
        <div className="flex items-center justify-between pt-1 border-t border-dashed">
          <div className="flex items-center gap-2">
            <Button
              variant="ghost"
              size="sm"
              onClick={handleAttachClick}
              disabled={disabled}
              className="h-7 px-2 text-xs text-muted-foreground hover:text-foreground"
            >
              <Paperclip className="h-3.5 w-3.5 mr-1" />
              {t('quickInput.attach')}
            </Button>
            {row.images.length > 0 && (
              <span className="text-xs text-muted-foreground">
                {t('quickInput.imageCount', { count: row.images.length })}
              </span>
            )}
          </div>
          <div className="flex items-center gap-2">
            <div className="w-36">
              <BranchCombobox
                branches={branches}
                value={row.branch}
                onChange={(value) => onChange(row.id, 'branch', value)}
                disabled={disabled}
                isLoading={branchesLoading}
              />
            </div>
            {showRemove && (
              <Button
                variant="ghost"
                size="icon"
                onClick={() => onRemove(row.id)}
                disabled={disabled}
                className="h-7 w-7 opacity-0 group-hover:opacity-100 transition-opacity"
              >
                <X className="h-4 w-4" />
              </Button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

export function QuickTaskInput({
  projectId,
  className,
  defaultCollapsed = false,
}: QuickTaskInputProps) {
  const { t } = useTranslation(['tasks', 'common']);
  const { createTask, createAndStart } = useTaskMutations(projectId);
  const { system, profiles, loading: userSystemLoading } = useUserSystem();
  const { upload } = useImageUpload();

  // Use a counter for generating unique IDs
  const idCounter = useRef(1);
  const generateId = () => `task-${idCounter.current++}`;

  const [isCollapsed, setIsCollapsed] = useState(defaultCollapsed);
  const [rows, setRows] = useState<TaskRow[]>(() => [
    { id: generateId(), prompt: '', branch: null, images: [] },
  ]);
  const [autoStart, setAutoStart] = useState(true);
  const [executorProfileId, setExecutorProfileId] =
    useState<ExecutorProfileId | null>(null);
  const [submittingCount, setSubmittingCount] = useState(0);
  const [newRowId, setNewRowId] = useState<string | null>(null);
  
  // Agent Swarm mode state
  const [executionMode, setExecutionMode] = useState<ExecutionMode>('single');
  const [maxParallelWorkers, setMaxParallelWorkers] = useState(3);
  const [reviewerCount, setReviewerCount] = useState(3);
  
  const isSwarmMode = executionMode === 'swarm';
  
  // When swarm mode is selected, force autoStart to be true
  // (swarm mode doesn't make sense without starting execution)
  const handleExecutionModeChange = useCallback((mode: ExecutionMode) => {
    setExecutionMode(mode);
    if (mode === 'swarm') {
      setAutoStart(true);
    }
  }, []);
  
  // When autoStart is toggled off, reset to single mode
  const handleAutoStartChange = useCallback((checked: boolean) => {
    setAutoStart(checked);
    if (!checked && isSwarmMode) {
      setExecutionMode('single');
    }
  }, [isSwarmMode]);

  // Set default executor profile when config loads
  useEffect(() => {
    if (system.config?.executor_profile && !executorProfileId) {
      setExecutorProfileId(system.config.executor_profile);
    }
  }, [system.config?.executor_profile, executorProfileId]);

  const { data: projectRepos = [] } = useProjectRepos(projectId);
  const { configs: repoBranchConfigs, isLoading: branchesLoading } =
    useRepoBranchSelection({
      repos: projectRepos,
      enabled: projectRepos.length > 0,
    });

  // Get default branch when configs load
  const defaultBranch = useMemo(() => {
    if (repoBranchConfigs.length > 0 && repoBranchConfigs[0].targetBranch) {
      return repoBranchConfigs[0].targetBranch;
    }
    return null;
  }, [repoBranchConfigs]);

  // Get branches from the first repo (for single-repo projects)
  const branches = useMemo(() => {
    if (repoBranchConfigs.length > 0) {
      return repoBranchConfigs[0].branches;
    }
    return [];
  }, [repoBranchConfigs]);

  // Get first repo ID
  const repoId = useMemo(() => {
    if (projectRepos.length > 0) {
      return projectRepos[0].id;
    }
    return null;
  }, [projectRepos]);

  // Initialize branch for first row when default branch is available
  useEffect(() => {
    if (defaultBranch && rows.length > 0 && rows[0].branch === null) {
      setRows((prev) =>
        prev.map((row, idx) =>
          idx === 0 && row.branch === null
            ? { ...row, branch: defaultBranch }
            : row
        )
      );
    }
  }, [defaultBranch, rows]);

  const isSubmitting = submittingCount > 0;
  
  // In swarm mode, autoStart is always effectively true
  const effectiveAutoStart = isSwarmMode ? true : autoStart;

  const validRows = useMemo(() => {
    return rows.filter((row) => {
      if (!row.prompt.trim()) return false;
      // In single agent mode with autostart, we need a branch
      // In swarm mode, branch is still used but we use the default branch if not set
      if (effectiveAutoStart && !isSwarmMode && !row.branch) return false;
      return true;
    });
  }, [rows, effectiveAutoStart, isSwarmMode]);

  const canSubmit = useMemo(() => {
    if (validRows.length === 0) return false;
    // In swarm mode, we don't need an executor profile - the swarm manages execution
    if (effectiveAutoStart && !isSwarmMode && !executorProfileId) return false;
    // In single agent mode with autostart, we need a repo
    if (effectiveAutoStart && !isSwarmMode && !repoId) return false;
    // In swarm mode, we need a repo for the workers
    if (isSwarmMode && !repoId) return false;
    return true;
  }, [validRows, effectiveAutoStart, isSwarmMode, executorProfileId, repoId]);

  const handleRowChange = useCallback(
    (id: string, field: 'prompt' | 'branch', value: string) => {
      setRows((prev) =>
        prev.map((row) => (row.id === id ? { ...row, [field]: value } : row))
      );
    },
    []
  );

  const handleImagesChange = useCallback(
    (id: string, images: ImageResponse[]) => {
      setRows((prev) =>
        prev.map((row) => (row.id === id ? { ...row, images } : row))
      );
    },
    []
  );

  const handleAddRow = useCallback(() => {
    const newId = generateId();
    setRows((prev) => [
      ...prev,
      { id: newId, prompt: '', branch: defaultBranch, images: [] },
    ]);
    setNewRowId(newId);
  }, [defaultBranch]);

  const handleRemoveRow = useCallback((id: string) => {
    setRows((prev) => {
      if (prev.length <= 1) return prev;
      return prev.filter((row) => row.id !== id);
    });
  }, []);

  const handleSubmit = useCallback(async () => {
    if (!canSubmit || isSubmitting) return;

    setSubmittingCount(validRows.length);

    const results: { success: boolean; row: TaskRow }[] = [];

    for (const row of validRows) {
      const imageIds =
        row.images.length > 0 ? row.images.map((img) => img.id) : null;

      const task = {
        project_id: projectId,
        title: row.prompt.trim(),
        description: null,
        status: null,
        parent_workspace_id: null,
        image_ids: imageIds,
        metadata: null,
        // Mark as epic task for swarm mode
        is_epic: isSwarmMode ? true : null,
        complexity: isSwarmMode ? ('epic' as const) : null,
      };

      try {
        if (isSwarmMode && repoId) {
          // Swarm mode: create epic task and start swarm execution
          const createdTask = await createTask.mutateAsync(task);
          if (createdTask) {
            try {
              // Create swarm execution
              const response = await fetch('/api/swarms', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                  epic_task_id: createdTask.id,
                  reviewer_count: reviewerCount,
                  max_parallel_workers: maxParallelWorkers,
                }),
              });
              if (!response.ok) {
                const errorText = await response.text();
                console.error('Failed to create swarm execution:', response.status, errorText);
                alert(`Epic task created, but swarm execution failed: ${errorText}\n\nPlease configure a planner agent in Settings → Agents.`);
              } else {
                const swarmExecution = await response.json();
                // Generate plan
                const planResponse = await fetch(`/api/swarms/${swarmExecution.id}/plan`, {
                  method: 'POST',
                });
                if (!planResponse.ok) {
                  const planError = await planResponse.text();
                  console.error('Failed to generate plan:', planError);
                  alert(`Swarm created, but plan generation failed: ${planError}`);
                } else {
                  // Execute plan
                  const executeResponse = await fetch(`/api/swarms/${swarmExecution.id}/execute`, {
                    method: 'POST',
                  });
                  if (!executeResponse.ok) {
                    const execError = await executeResponse.text();
                    console.error('Failed to execute plan:', execError);
                    alert(`Plan generated, but execution failed: ${execError}`);
                  }
                }
              }
            } catch (err) {
              console.error('Failed to start swarm execution:', err);
              alert(`Failed to start swarm execution: ${err}`);
            }
          }
          results.push({ success: true, row });
        } else if (autoStart && repoId) {
          // Single agent mode
          const repos = [{ repo_id: repoId, target_branch: row.branch! }];
          await createAndStart.mutateAsync({
            task,
            executor_profile_id: executorProfileId!,
            repos,
          });
          results.push({ success: true, row });
        } else {
          await createTask.mutateAsync(task);
          results.push({ success: true, row });
        }
      } catch (error) {
        console.error('Failed to create task:', error);
        results.push({ success: false, row });
      }
      setSubmittingCount((c) => c - 1);
    }

    // Reset rows that were successfully submitted
    const successfulIds = results
      .filter((r) => r.success)
      .map((r) => r.row.id);
    setRows((prev) => {
      const remaining = prev.filter((row) => !successfulIds.includes(row.id));
      if (remaining.length === 0) {
        return [{ id: generateId(), prompt: '', branch: defaultBranch, images: [] }];
      }
      return remaining;
    });
  }, [
    canSubmit,
    isSubmitting,
    validRows,
    projectId,
    autoStart,
    repoId,
    executorProfileId,
    createAndStart,
    createTask,
    defaultBranch,
    isSwarmMode,
    maxParallelWorkers,
    reviewerCount,
  ]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent, rowId: string) => {
      // Cmd/Ctrl + Enter to submit
      if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        handleSubmit();
        return;
      }
      // Backspace on empty row removes it (only if cursor is at start)
      if (e.key === 'Backspace') {
        const row = rows.find((r) => r.id === rowId);
        const target = e.target as HTMLTextAreaElement;
        if (row && !row.prompt && rows.length > 1 && target.selectionStart === 0) {
          e.preventDefault();
          handleRemoveRow(rowId);
        }
      }
    },
    [rows, handleSubmit, handleRemoveRow]
  );

  const loading = branchesLoading || userSystemLoading;

  // Collapsed state - show a compact bar
  if (isCollapsed) {
    return (
      <div className={cn('w-full', className)}>
        <Button
          variant="outline"
          onClick={() => setIsCollapsed(false)}
          className="w-full justify-start gap-2 h-10 text-muted-foreground hover:text-foreground"
        >
          <MessageSquarePlus className="h-4 w-4" />
          <span className="text-sm">{t('quickInput.placeholderFirst')}</span>
          <ChevronDown className="h-4 w-4 ml-auto" />
        </Button>
      </div>
    );
  }

  return (
    <div className={cn('w-full max-w-2xl mx-auto', className)}>
      {/* Collapse button */}
      {defaultCollapsed && (
        <div className="flex justify-end mb-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setIsCollapsed(true)}
            className="h-6 px-2 text-xs text-muted-foreground"
          >
            <ChevronUp className="h-3 w-3 mr-1" />
            {t('quickInput.collapse')}
          </Button>
        </div>
      )}

      <div className="space-y-3">
        {/* Task rows */}
        {rows.map((row, idx) => (
          <TaskRowInput
            key={row.id}
            row={row}
            branches={branches}
            branchesLoading={branchesLoading}
            onChange={handleRowChange}
            onImagesChange={handleImagesChange}
            onRemove={handleRemoveRow}
            onKeyDown={handleKeyDown}
            showRemove={rows.length > 1}
            disabled={isSubmitting}
            autoFocus={row.id === newRowId}
            placeholder={
              idx === 0
                ? t('quickInput.placeholderFirst')
                : t('quickInput.placeholderMore')
            }
            upload={upload}
          />
        ))}

        {/* Add row button */}
        <Button
          variant="ghost"
          size="sm"
          onClick={handleAddRow}
          disabled={isSubmitting}
          className="w-full h-9 text-xs text-muted-foreground hover:text-foreground border border-dashed"
        >
          <Plus className="h-3.5 w-3.5 mr-1.5" />
          {t('quickInput.addTask')}
        </Button>
      </div>

      {/* Bottom toolbar */}
      <div className="mt-4 p-3 rounded-lg border bg-muted/30">
        {/* Start toggle and submit button row */}
        <div className="flex items-center justify-between gap-3">
          <div className="flex items-center gap-2">
            <Switch
              id="quick-autostart"
              checked={effectiveAutoStart}
              onCheckedChange={handleAutoStartChange}
              disabled={isSubmitting || loading || isSwarmMode}
              className="data-[state=checked]:bg-gray-900 dark:data-[state=checked]:bg-gray-100"
            />
            <Label
              htmlFor="quick-autostart"
              className={cn(
                'text-sm cursor-pointer text-muted-foreground',
                isSwarmMode && 'opacity-70'
              )}
            >
              {t('taskFormDialog.startLabel')}
              {isSwarmMode && (
                <span className="ml-1 text-xs text-purple-600 dark:text-purple-400">
                  (required)
                </span>
              )}
            </Label>
          </div>

          <div className="flex items-center gap-2">
            {validRows.length > 1 && (
              <span className="text-xs text-muted-foreground">
                {t('quickInput.taskCount', { count: validRows.length })}
              </span>
            )}
            <Button
              onClick={handleSubmit}
              disabled={!canSubmit || isSubmitting}
              size="sm"
              className={cn(
                'gap-2',
                isSwarmMode && 'bg-purple-600 hover:bg-purple-700'
              )}
            >
              {isSubmitting ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin" />
                  {isSwarmMode
                    ? t('quickInput.launchingSwarm', { count: submittingCount })
                    : t('quickInput.creating', { count: submittingCount })}
                </>
              ) : (
                <>
                  {isSwarmMode ? (
                    <Users className="h-4 w-4" />
                  ) : (
                    <Send className="h-4 w-4" />
                  )}
                  {isSwarmMode
                    ? t('quickInput.launchSwarm')
                    : autoStart
                      ? t('quickInput.start')
                      : t('quickInput.create')}
                  {validRows.length > 1 && ` (${validRows.length})`}
                </>
              )}
            </Button>
          </div>
        </div>

        {/* Execution Mode Selector */}
        {autoStart && (
          <div className="mt-3 pt-3 border-t space-y-3">
            <Tabs
              value={executionMode}
              onValueChange={(v) => handleExecutionModeChange(v as ExecutionMode)}
              className="w-full"
            >
              <TabsList className="grid w-full grid-cols-2 h-9">
                <TabsTrigger
                  value="single"
                  className="flex items-center gap-1.5 text-xs"
                  disabled={isSubmitting}
                >
                  <User className="h-3.5 w-3.5" />
                  {t('quickInput.singleAgent')}
                </TabsTrigger>
                <TabsTrigger
                  value="swarm"
                  className="flex items-center gap-1.5 text-xs"
                  disabled={isSubmitting}
                >
                  <Users className="h-3.5 w-3.5" />
                  {t('quickInput.agentSwarm')}
                  <Badge variant="secondary" className="ml-1 text-[10px] px-1 py-0">
                    Beta
                  </Badge>
                </TabsTrigger>
              </TabsList>
            </Tabs>

            {/* Swarm description */}
            {isSwarmMode && (
              <div className="p-2.5 bg-purple-50 dark:bg-purple-900/20 rounded-lg border border-purple-200 dark:border-purple-800">
                <div className="flex items-start gap-2">
                  <Crown className="h-4 w-4 text-purple-600 mt-0.5 flex-shrink-0" />
                  <div className="text-xs">
                    <p className="font-medium text-purple-900 dark:text-purple-100">
                      {t('quickInput.swarmTitle')}
                    </p>
                    <p className="text-purple-700 dark:text-purple-300 mt-0.5">
                      {t('quickInput.swarmDescription')}
                    </p>
                  </div>
                </div>
              </div>
            )}

            {/* Swarm Configuration */}
            {isSwarmMode && (
              <div className="space-y-3 p-3 border rounded-lg bg-background">
                <div className="flex items-center gap-2">
                  <Zap className="h-3.5 w-3.5 text-yellow-600" />
                  <Label className="text-xs font-medium">
                    {t('quickInput.swarmConfig')}
                  </Label>
                </div>

                {/* Max Parallel Workers */}
                <div className="space-y-1.5">
                  <div className="flex items-center justify-between">
                    <Label className="text-xs">{t('quickInput.parallelWorkers')}</Label>
                    <span className="text-xs font-medium">{maxParallelWorkers}</span>
                  </div>
                  <Slider
                    value={[maxParallelWorkers]}
                    onValueChange={([v]) => setMaxParallelWorkers(v)}
                    min={1}
                    max={10}
                    step={1}
                    disabled={isSubmitting}
                    className="w-full"
                  />
                  <p className="text-[10px] text-muted-foreground">
                    {t('quickInput.parallelWorkersHint')}
                  </p>
                </div>

                {/* Reviewer Count */}
                <div className="space-y-1.5">
                  <div className="flex items-center justify-between">
                    <Label className="text-xs">{t('quickInput.reviewers')}</Label>
                    <span className="text-xs font-medium">{reviewerCount}</span>
                  </div>
                  <Slider
                    value={[reviewerCount]}
                    onValueChange={([v]) => setReviewerCount(v)}
                    min={1}
                    max={7}
                    step={1}
                    disabled={isSubmitting}
                    className="w-full"
                  />
                  <p className="text-[10px] text-muted-foreground">
                    {t('quickInput.reviewersHint', {
                      approvals: Math.floor((reviewerCount * 2) / 3) + 1,
                      faulty: Math.floor((reviewerCount - 1) / 3),
                    })}
                  </p>
                </div>
              </div>
            )}
          </div>
        )}

        {/* Executor selector - shown when autoStart is enabled and NOT in swarm mode */}
        {autoStart && !isSwarmMode && !loading && profiles && (
          <div className="mt-3 pt-3 border-t">
            <ExecutorProfileSelector
              profiles={profiles}
              selectedProfile={executorProfileId}
              onProfileSelect={setExecutorProfileId}
              disabled={isSubmitting}
              showLabel={false}
              className="flex items-center gap-2 flex-row"
              itemClassName="flex-1 min-w-0"
            />
          </div>
        )}
      </div>

      {/* Hint text */}
      <p className="text-center text-xs text-muted-foreground mt-3">
        {isSwarmMode ? t('quickInput.hintSwarm') : t('quickInput.hintMulti')}
      </p>
    </div>
  );
}
