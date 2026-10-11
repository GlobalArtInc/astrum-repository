

import React, { useState, useMemo, useEffect, useCallback } from 'react';
import { Box, Flex, Text, TextField, Badge, Skeleton } from '@radix-ui/themes';
import { Clock, ListTodo, RefreshCw, CheckCircle, XCircle, AlertCircle, Play, Square } from 'lucide-react';
import { ActionIcons } from '../../../../shared/icons/action-icons';
import { ExtJS } from '../../../../../../interface/ExtJS';
import plural from '../../../../../../interface/plural';

import {
  EntityTable,
  FilterSidebar,
  EmptyState,
  HelpSection,
  useToast,
  type TableColumn,
  type FilterSection,
} from '../../../../shared';
import { DeleteConfirmationModal } from '../../../../shared/modals/DeleteConfirmationModal';
import { useTasksApi } from './useTasksApi';
import { getTaskCategoryLabel } from './taskTypeDescriptions';
import { getTaskScheduleLabel, getTaskStatusBadgeColor, getTaskStatusLabel } from './taskLabels';
import { usePolling, POLL_INTERVAL_MS, POST_ACTION_POLL_COUNT } from './usePolling';
import {
  Task,
  TasksListProps,
  TaskStatus,
  formatDate,
  isActiveStatus,
} from './types';

import './TasksList.scss';
import TasksStrings from '../../../../constants/pages/settings/system/TasksStrings';
import CommonStrings from '../../../../constants/CommonStrings';

type SortField = 'name' | 'typeName' | 'status' | 'schedule' | 'nextRun' | 'lastRun';

function getStatusIcon(status: TaskStatus) {
  switch (status) {
    case 'RUNNING': return <RefreshCw size={14} className="tasks-list__status-icon tasks-list__status-icon--running" />;
    case 'OK': return <CheckCircle size={14} className="tasks-list__status-icon tasks-list__status-icon--ok" />;
    case 'WAITING': return <Clock size={14} className="tasks-list__status-icon tasks-list__status-icon--waiting" />;
    case 'BLOCKED': return <AlertCircle size={14} className="tasks-list__status-icon tasks-list__status-icon--blocked" />;
    default: return <XCircle size={14} className="tasks-list__status-icon tasks-list__status-icon--error" />;
  }
}

function getTaskCategory(typeId: string): string {
  if (typeId.startsWith('repository.maven')) return 'Maven';
  if (typeId.startsWith('repository.docker')) return 'Docker';
  if (typeId.startsWith('repository.npm')) return 'npm';
  if (typeId.startsWith('repository.')) return 'Repository';
  if (typeId.startsWith('blobstore.') || typeId.startsWith('db.') || typeId.startsWith('h2.') ||
      typeId.startsWith('security.') || typeId.startsWith('usertoken.')) return 'Admin';
  if (typeId.startsWith('healthcheck') || typeId.startsWith('malicious')) return 'Health Check';
  if (typeId.startsWith('assetBlob.')) return 'Cleanup';
  if (typeId.startsWith('tags.')) return 'Tags';
  return 'Other';
}

export function TasksList({ onSelect, onCreate }: TasksListProps) {
  const { fetchTasks, runTask, stopTask, deleteTask, error: apiError } = useTasksApi();
  const toast = useToast();

  const [tasks, setTasks] = useState<Task[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState('');
  const [statusFilter, setStatusFilter] = useState<string[]>([]);
  const [typeFilter, setTypeFilter] = useState<string[]>([]);
  const [sortField, setSortField] = useState<SortField>('name');
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('asc');
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [taskToDelete, setTaskToDelete] = useState<Task | null>(null);
  const [actionLoading, setActionLoading] = useState<string | null>(null);
  // Post-action poll window (ASTRUM-53525): keeps the list refreshing for a few
  // ticks after Run/Stop even when no row is RUNNING yet (bridges WAITING→RUNNING).
  const [pollsLeft, setPollsLeft] = useState(0);

  const canDelete = ExtJS.checkPermission('astrum:tasks:delete');
  const canRun = ExtJS.checkPermission('astrum:tasks:start');
  // Stopping requires astrum:tasks:stop, distinct from the start permission (matches Classic).
  const canStop = ExtJS.checkPermission('astrum:tasks:stop');

  useEffect(() => {
    const loadTasks = async () => {
      setLoading(true);
      try {
        const data = await fetchTasks();
        setTasks(data);
      } catch {
        // error is set by the API hook
      } finally {
        setLoading(false);
      }
    };
    loadTasks();
  }, [fetchTasks]);

  // Live status polling (ASTRUM-53525): refresh the list so Status / Last Run /
  // Last Result reflect server state without a manual reload. Poll only while a
  // visible row is active (RUNNING/BLOCKED) or inside the post-action window —
  // once everything is stable/terminal the list goes quiet.
  const hasActiveTask = useMemo(() => tasks.some((t) => isActiveStatus(t.status)), [tasks]);
  const pollingEnabled = hasActiveTask || pollsLeft > 0;

  const pollTasks = useCallback(async () => {
    const data = await fetchTasks();
    setTasks(data);
    // While anything is active, isActiveStatus keeps polling; otherwise burn down
    // the post-action window so an all-stable list stops polling.
    setPollsLeft((remaining) => (data.some((t) => isActiveStatus(t.status)) ? 0 : Math.max(0, remaining - 1)));
  }, [fetchTasks]);

  // pollOnEnable is false: the initial load above already fetched once and
  // Run/Stop trigger their own immediate pollNow(), so there is no duplicate fetch.
  const { pollNow: pollTasksNow } = usePolling(pollTasks, {
    intervalMs: POLL_INTERVAL_MS,
    enabled: pollingEnabled,
    pollOnEnable: false,
  });

  const { statusCounts, typeCounts } = useMemo(() => {
    const sc = new Map<string, number>();
    const tc = new Map<string, number>();
    tasks.forEach((t) => {
      const status = t.enabled ? t.status : 'DISABLED';
      sc.set(status, (sc.get(status) || 0) + 1);
      const cat = getTaskCategory(t.typeId);
      tc.set(cat, (tc.get(cat) || 0) + 1);
    });
    return { statusCounts: sc, typeCounts: tc };
  }, [tasks]);

  const hasActiveFilters = searchQuery.trim() !== '' || statusFilter.length > 0 || typeFilter.length > 0;
  const activeFilterCount = (searchQuery.trim() !== '' ? 1 : 0) + statusFilter.length + typeFilter.length;

  const filteredTasks = useMemo(() => {
    let result = [...tasks];

    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      result = result.filter((t) =>
        t.name.toLowerCase().includes(q) ||
        t.typeName.toLowerCase().includes(q) ||
        t.statusDescription.toLowerCase().includes(q)
      );
    }

    if (statusFilter.length > 0) {
      result = result.filter((t) => {
        const effectiveStatus = t.enabled ? t.status : 'DISABLED';
        return statusFilter.includes(effectiveStatus);
      });
    }

    if (typeFilter.length > 0) {
      result = result.filter((t) => typeFilter.includes(getTaskCategory(t.typeId)));
    }

    result.sort((a, b) => {
      let cmp = 0;
      switch (sortField) {
        case 'name': cmp = a.name.localeCompare(b.name); break;
        case 'typeName': cmp = a.typeName.localeCompare(b.typeName); break;
        case 'status': cmp = a.status.localeCompare(b.status); break;
        case 'schedule': cmp = (a.schedule || '').localeCompare(b.schedule || ''); break;
        case 'nextRun':
          if (!(a.nextRun || b.nextRun)) cmp = 0;
          else if (!a.nextRun) cmp = 1;
          else if (!b.nextRun) cmp = -1;
          else cmp = new Date(a.nextRun).getTime() - new Date(b.nextRun).getTime();
          break;
        case 'lastRun':
          if (!(a.lastRun || b.lastRun)) cmp = 0;
          else if (!a.lastRun) cmp = 1;
          else if (!b.lastRun) cmp = -1;
          else cmp = new Date(a.lastRun).getTime() - new Date(b.lastRun).getTime();
          break;
      }
      return sortDirection === 'asc' ? cmp : -cmp;
    });

    return result;
  }, [tasks, searchQuery, statusFilter, typeFilter, sortField, sortDirection]);

  const handleSort = useCallback((columnId: string) => {
    const field = columnId as SortField;
    if (sortField === field) {
      setSortDirection((prev) => (prev === 'asc' ? 'desc' : 'asc'));
    } else {
      setSortField(field);
      setSortDirection('asc');
    }
  }, [sortField]);

  const handleFilterChange = useCallback((sectionId: string, value: string | string[]) => {
    if (sectionId === 'status') setStatusFilter(value as string[]);
    else if (sectionId === 'type') setTypeFilter(value as string[]);
  }, []);

  const handleClearFilters = useCallback(() => {
    setSearchQuery('');
    setStatusFilter([]);
    setTypeFilter([]);
  }, []);

  const handleRowClick = useCallback((task: Task) => {
    onSelect(task.id);
  }, [onSelect]);

  // Run/Stop reconciled with polling (ASTRUM-53525): open the post-action window
  // and fire a single immediate refresh poll, then let the interval carry the row
  // through WAITING → RUNNING → terminal. Replaces the old one-shot refetch, so
  // there is never a duplicate fetch racing the poll.
  const handleRun = useCallback(async (e: React.MouseEvent, task: Task) => {
    e.stopPropagation();
    setActionLoading(task.id);
    try {
      await runTask(task.id);
      toast.success(TasksStrings.LIST.TASK_STARTED(task.name));
      setPollsLeft(POST_ACTION_POLL_COUNT);
      pollTasksNow();
    } catch {
      toast.error(TasksStrings.LIST.FAILED_RUN_TASK(task.name));
    } finally {
      setActionLoading(null);
    }
  }, [runTask, toast, pollTasksNow]);

  const handleStop = useCallback(async (e: React.MouseEvent, task: Task) => {
    e.stopPropagation();
    setActionLoading(task.id);
    try {
      await stopTask(task.id);
      toast.success(TasksStrings.LIST.TASK_STOPPED(task.name));
      setPollsLeft(POST_ACTION_POLL_COUNT);
      pollTasksNow();
    } catch {
      toast.error(TasksStrings.LIST.FAILED_STOP_TASK(task.name));
    } finally {
      setActionLoading(null);
    }
  }, [stopTask, toast, pollTasksNow]);

  const handleDeleteClick = useCallback((e: React.MouseEvent, task: Task) => {
    e.stopPropagation();
    setTaskToDelete(task);
    setDeleteDialogOpen(true);
  }, []);

  const handleDeleteConfirm = useCallback(async () => {
    if (!taskToDelete) return;
    setActionLoading(taskToDelete.id);
    try {
      await deleteTask(taskToDelete.id);
      toast.success(TasksStrings.LIST.TASK_DELETED(taskToDelete.name));
      setTasks((prev) => prev.filter((t) => t.id !== taskToDelete.id));
      setDeleteDialogOpen(false);
      setTaskToDelete(null);
    } catch {
      toast.error(TasksStrings.LIST.FAILED_DELETE_TASK(taskToDelete.name));
      setDeleteDialogOpen(false);
    } finally {
      setActionLoading(null);
    }
  }, [taskToDelete, deleteTask, toast]);

  const filterSections = useMemo<FilterSection[]>(() => [
    {
      id: 'status',
      label: CommonStrings.LABELS.STATUS,
      type: 'checkbox',
      options: Array.from(statusCounts.entries())
        .sort(([a], [b]) => a.localeCompare(b))
        .map(([value, count]) => ({ value, label: getTaskStatusLabel(value), count })),
      value: statusFilter,
      defaultExpanded: true,
    },
    {
      id: 'type',
      label: TasksStrings.LIST.CATEGORY,
      type: 'checkbox',
      options: Array.from(typeCounts.entries())
        .sort(([a], [b]) => a.localeCompare(b))
        .map(([value, count]) => ({ value, label: getTaskCategoryLabel(value), count })),
      value: typeFilter,
      defaultExpanded: true,
    },
  ], [statusCounts, typeCounts, statusFilter, typeFilter]);

  const hasDescriptions = useMemo(() => tasks.some((t) => !!t.statusDescription), [tasks]);

  const columns = useMemo<TableColumn<Task>[]>(() => [
    {
      id: 'name',
      header: CommonStrings.LABELS.NAME,
      accessor: (task) => (
        <Flex align="center" gap="2">
          <Text weight="medium">{task.name}</Text>
          {!task.enabled && <Badge size="1" color="gray">{TasksStrings.LIST.DISABLED}</Badge>}
        </Flex>
      ),
      sortable: true,
      width: '200px',
    },
    {
      id: 'typeName',
      header: CommonStrings.LABELS.TYPE,
      accessor: (task) => <Text size="2">{task.typeName}</Text>,
      sortable: true,
      width: '130px',
    },
    // The server reports a description only for some tasks while they run; without one the column stays empty
    ...(hasDescriptions ? [{
      id: 'description',
      header: CommonStrings.LABELS.DESCRIPTION,
      accessor: (task) => (
        <Text size="2" color="gray" style={{ maxWidth: 250, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', display: 'block' }}>
          {task.statusDescription || '-'}
        </Text>
      ),
      sortable: true,
      width: '250px',
    } as TableColumn<Task>] : []),
    {
      id: 'status',
      header: CommonStrings.LABELS.STATUS,
      accessor: (task) => (
        <Flex align="center" gap="2">
          {getStatusIcon(task.status)}
          <Badge
            size="1"
            color={getTaskStatusBadgeColor(task.status)}
            variant={task.status === 'BLOCKED' ? 'solid' : 'soft'}
            className={task.status === 'RUNNING' ? 'tasks-list__status-badge--running' : ''}
          >
            {getTaskStatusLabel(task.status)}
          </Badge>
        </Flex>
      ),
      sortable: true,
      width: '120px',
    },
    {
      id: 'schedule',
      header: TasksStrings.LIST.SCHEDULE,
      accessor: (task) => (
        <Text size="2">{getTaskScheduleLabel(task.schedule)}</Text>
      ),
      sortable: true,
      width: '90px',
    },
    {
      id: 'nextRun',
      header: TasksStrings.LIST.NEXT_RUN,
      accessor: (task) => <Text size="2">{formatDate(task.nextRun)}</Text>,
      sortable: true,
      width: '140px',
    },
    {
      id: 'lastRun',
      header: TasksStrings.LIST.LAST_RUN,
      accessor: (task) => <Text size="2">{formatDate(task.lastRun)}</Text>,
      sortable: true,
      width: '140px',
    },
    {
      id: 'lastResult',
      header: TasksStrings.LIST.LAST_RESULT,
      accessor: (task) => <Text size="2">{task.lastRunResult || '-'}</Text>,
      width: '80px',
    },
    {
      id: 'actions',
      header: '',
      accessor: (task) => (
        <Flex align="center" gap="1" className="tasks-list__actions" onClick={(e) => e.stopPropagation()}>
          {canRun && task.runnable && task.status !== 'RUNNING' && (
            <button
              type="button"
              className="tasks-list__action-btn tasks-list__action-btn--run"
              onClick={(e) => handleRun(e, task)}
              disabled={actionLoading === task.id}
              aria-label={TasksStrings.LIST.RUN_TASK(task.name)}
              title={TasksStrings.LIST.RUN_NOW}
            >
              <Play size={14} />
            </button>
          )}
          {canStop && task.stoppable && task.status === 'RUNNING' && (
            <button
              type="button"
              className="tasks-list__action-btn tasks-list__action-btn--stop"
              onClick={(e) => handleStop(e, task)}
              disabled={actionLoading === task.id}
              aria-label={TasksStrings.LIST.STOP_TASK(task.name)}
              title={TasksStrings.LIST.STOP_TITLE}
            >
              <Square size={14} />
            </button>
          )}
          {canDelete && (
            <button
              type="button"
              className="tasks-list__action-btn tasks-list__action-btn--delete"
              onClick={(e) => handleDeleteClick(e, task)}
              aria-label={TasksStrings.LIST.DELETE_TASK(task.name)}
              title={CommonStrings.ACTIONS.DELETE}
            >
              <ActionIcons.Delete size={14} />
            </button>
          )}
        </Flex>
      ),
      width: '90px',
      align: 'right',
    },
  ], [canRun, canStop, canDelete, actionLoading, handleRun, handleStop, handleDeleteClick, hasDescriptions]);

  const emptyState = useMemo(() => {
    const hasFilters = searchQuery || statusFilter.length > 0 || typeFilter.length > 0;
    if (hasFilters) {
      return (
        <EmptyState
          icon={ListTodo}
          title={TasksStrings.LIST.NO_MATCHING_TASKS}
          description={TasksStrings.LIST.NO_TASKS_MATCH_CURRENT_FILTERS}
          action={{ label: CommonStrings.ACTIONS.CLEAR_FILTERS, onClick: handleClearFilters }}
        />
      );
    }
    return (
      <EmptyState
        icon={ListTodo}
        title={TasksStrings.LIST.NO_TASKS}
        description={TasksStrings.LIST.CREATE_FIRST_SCHEDULED_TASK_AUTOMATE}
        action={onCreate ? { label: TasksStrings.LIST.CREATE_TASK, onClick: onCreate, icon: ActionIcons.Add } : undefined}
        secondaryAction={{
          label: TasksStrings.LIST.LEARN_MORE_ABOUT_TASKS,
          href: 'http://links.globalart.dev/products/astrum/docs/tasks',
        }}
        tip={TasksStrings.LIST.SCHEDULED_TASKS_AUTOMATE_COMMON_OPERATIONS}
      />
    );
  }, [searchQuery, statusFilter, typeFilter, handleClearFilters, onCreate]);

  return (
    <Flex className="tasks-list" gap="4" data-testid="tasks-list">
      <Box className="tasks-list__sidebar">
        <FilterSidebar
          sections={filterSections}
          onFilterChange={handleFilterChange}
          onClear={hasActiveFilters ? handleClearFilters : undefined}
          disabled={loading}
        />
        {hasActiveFilters && (
          <Box className="tasks-list__filter-badge">
            <Badge size="2" color="blue" variant="soft">
              {plural(TasksStrings.LIST.FILTERS_ACTIVE, activeFilterCount)}
            </Badge>
          </Box>
        )}
      </Box>

      <Box className="tasks-list__main">
        <Box className="tasks-list__search-container">
          <TextField.Root
            placeholder={TasksStrings.LIST.SEARCH_TASKS_NAME_TYPE_STATUS_PLACEHOLDER}
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="tasks-list__search-input"
            data-testid="tasks-search"
            autoComplete="off"
          >
            <TextField.Slot>
              <ActionIcons.Search size={16} />
            </TextField.Slot>
            {searchQuery && (
              <TextField.Slot>
                <button
                  type="button"
                  className="tasks-list__clear-btn"
                  onClick={() => setSearchQuery('')}
                  aria-label={TasksStrings.LIST.CLEAR_SEARCH_ARIA}
                >
                  <ActionIcons.Cancel size={14} />
                </button>
              </TextField.Slot>
            )}
          </TextField.Root>
        </Box>

        {loading && tasks.length === 0 ? (
          <Box className="tasks-list__skeleton">
            {Array.from({ length: 5 }).map((_, i) => (
              <Flex key={i} gap="3" align="center" px="3" py="2" data-testid="tasks-list-skeleton-row">
                <Skeleton width="20%" height="18px" />
                <Skeleton width="15%" height="18px" />
                <Skeleton width="12%" height="18px" />
                <Skeleton width="18%" height="18px" />
                <Skeleton width="15%" height="18px" />
                <Skeleton width="20%" height="18px" />
              </Flex>
            ))}
          </Box>
        ) : (
          <EntityTable<Task>
            data={filteredTasks}
            columns={columns}
            getRowKey={(task) => task.id}
            onRowClick={handleRowClick}
            loading={loading && tasks.length > 0}
            error={apiError || undefined}
            onRetry={async () => {
              setLoading(true);
              try {
                const data = await fetchTasks();
                setTasks(data);
              } finally {
                setLoading(false);
              }
            }}
            emptyState={emptyState}
            sortBy={sortField}
            sortDirection={sortDirection}
            onSort={handleSort}
            showRowArrow={true}
            clickable={true}
            ariaLabel={TasksStrings.LIST.TASKS_LIST_ARIA}
            className="tasks-list__table"
          />
        )}

        {!(loading || apiError ) && filteredTasks.length > 0 && (
          <Box className="tasks-list__summary">
            <Text size="2" color="gray">
              {TasksStrings.LIST.SHOWING(
                <Text as="span" weight="medium">{filteredTasks.length}</Text>,
                <Text as="span" weight="medium">{tasks.length}</Text>
              )}
            </Text>
          </Box>
        )}

        <HelpSection
          title={TasksStrings.LIST.ABOUT_SCHEDULED_TASKS}
          content={TasksStrings.LIST.ABOUT_SCHEDULED_TASKS_TEXT}
          docLink={{
            label: CommonStrings.ACTIONS.VIEW_DOCUMENTATION,
            href: 'http://links.globalart.dev/products/astrum/docs/tasks',
          }}
          className="tasks-list__help"
        />
      </Box>

      <DeleteConfirmationModal
        open={deleteDialogOpen}
        onClose={() => setDeleteDialogOpen(false)}
        onConfirm={handleDeleteConfirm}
        entityName={taskToDelete?.name}
        entityType="task"
        loading={actionLoading === taskToDelete?.id}
      >
        {taskToDelete && (
          <Flex direction="column" gap="1">
            <Flex gap="2">
              <Text size="2" color="gray">{TasksStrings.LIST.TYPE_LABEL}</Text>
              <Text size="2">{taskToDelete.typeName}</Text>
            </Flex>
            <Flex gap="2">
              <Text size="2" color="gray">{TasksStrings.LIST.SCHEDULE_LABEL}</Text>
              <Text size="2">{taskToDelete.schedule ? getTaskScheduleLabel(taskToDelete.schedule) : TasksStrings.LIST.NONE}</Text>
            </Flex>
            {taskToDelete.lastRun && (
              <Flex gap="2">
                <Text size="2" color="gray">{TasksStrings.LIST.LAST_RUN_LABEL}</Text>
                <Text size="2">
                  {formatDate(taskToDelete.lastRun)}
                  {taskToDelete.lastRunResult && ` (${taskToDelete.lastRunResult})`}
                </Text>
              </Flex>
            )}
          </Flex>
        )}
      </DeleteConfirmationModal>
    </Flex>
  );
}

export default TasksList;
