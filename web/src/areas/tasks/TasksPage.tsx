import { Button, Table } from '@radix-ui/themes';
import { ArrowDown, ArrowUp, ChevronRight, Play, Plus, Square, Trash2 } from 'lucide-react';
import { type ReactNode, useMemo, useState } from 'react';
import { Link, useNavigate, useSearchParams } from 'react-router';
import { api, path, useApi, useInvalidateApi } from '../../api';
import { useCan } from '../../auth/state';
import { useT } from '../../i18n';
import { describeError, ErrorState, Page, Tooltip, useToast } from '../../ui';
import { FilterPanel, SearchBox, sortList, useCollator } from '../security/shared/AdminList';
import list from '../security/shared/AdminList.module.css';
import {
  categoryKeyLabel,
  resultLabel,
  StateBadge,
  scheduleLabel,
  statusLabel,
  typeDescription,
  useLocalTime,
} from './components/bits';
import { DeleteTaskDialog } from './components/DeleteTaskDialog';
import {
  type Category,
  CREATE,
  categoryOf,
  countBy,
  DELETE,
  isActive,
  LIST_PAGE,
  matchesQuery,
  START,
  STOP,
  type StatusFilter,
  scheduleKind,
  statusOf,
  TASKS,
  type Task,
  taskHref,
  UI_TASKS,
} from './model';
import styles from './Tasks.module.css';

const STATUSES: StatusFilter[] = ['running', 'queued', 'waiting', 'disabled'];

function selected(params: URLSearchParams, name: string): string[] {
  return (params.get(name) ?? '').split(',').filter(Boolean);
}

export function useTaskActions() {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const refresh = () => Promise.all([invalidate(UI_TASKS), invalidate(TASKS)]);
  return {
    run: async (task: Pick<Task, 'id' | 'name'>) => {
      await api.post(`${TASKS}${path`/${task.id}/run`}`);
      toast.success(t('tasks.toasts.started', { name: task.name }));
      await refresh();
    },
    stop: async (task: Pick<Task, 'id' | 'name'>) => {
      await api.post(`${TASKS}${path`/${task.id}/stop`}`);
      toast.success(t('tasks.toasts.stopped', { name: task.name }));
      await refresh();
    },
    remove: async (task: Pick<Task, 'id' | 'name'>) => {
      await api.delete(`${TASKS}${path`/${task.id}`}`);
      toast.success(t('tasks.toasts.deleted', { name: task.name }));
      await refresh();
    },
  };
}

interface TaskColumn {
  id: string;
  header: string;
  width: number;
  cell(task: Task): ReactNode;
  sortValue?(task: Task): string | number;
}

function lastStarted(task: Task): string | null {
  return task.current?.startedAt ?? task.lastRun?.startedAt ?? null;
}

export default function TasksPage() {
  const t = useT();
  const time = useLocalTime();
  const toast = useToast();
  const navigate = useNavigate();
  const collator = useCollator();
  const actions = useTaskActions();
  const canCreate = useCan({ all: [CREATE] });
  const canStart = useCan(START);
  const canStop = useCan(STOP);
  const canDelete = useCan(DELETE);
  const [params, setParams] = useSearchParams();
  const [deleting, setDeleting] = useState<Task | null>(null);
  const [sort, setSort] = useState<{ column: string; direction: 'asc' | 'desc' }>({
    column: 'name',
    direction: 'asc',
  });
  const tasks = useApi<Task[]>(UI_TASKS, {
    refetchInterval: (query) => (query.state.data?.some(isActive) ? 2000 : 15000),
  });
  const query = params.get('q') ?? '';
  const chosen = { status: selected(params, 'status'), category: selected(params, 'category') };

  const update = (name: string, value: string) =>
    setParams(
      (previous) => {
        const next = new URLSearchParams(previous);
        if (value) {
          next.set(name, value);
        } else {
          next.delete(name);
        }
        return next;
      },
      { replace: true },
    );

  const attempt = async (work: () => Promise<void>) => {
    try {
      await work();
    } catch (error) {
      toast.error(describeError(t, error));
    }
  };

  const columns: TaskColumn[] = [
    {
      id: 'name',
      header: t('tasks.list.name'),
      width: 200,
      cell: (task) => <span className={styles.nameCell}>{task.name}</span>,
      sortValue: (task) => task.name,
    },
    {
      id: 'type',
      header: t('tasks.list.type'),
      width: 130,
      cell: (task) => task.typeName,
      sortValue: (task) => task.typeName,
    },
    {
      id: 'description',
      header: t('tasks.list.description'),
      width: 250,
      cell: (task) => <span className={styles.descriptionCell}>{typeDescription(t, task.type)}</span>,
      sortValue: (task) => typeDescription(t, task.type),
    },
    {
      id: 'status',
      header: t('tasks.list.status'),
      width: 120,
      cell: (task) => <StateBadge status={statusOf(task)} label={statusLabel(t, statusOf(task))} />,
      sortValue: (task) => statusLabel(t, statusOf(task)),
    },
    {
      id: 'schedule',
      header: t('tasks.list.schedule'),
      width: 90,
      cell: (task) => scheduleLabel(t, scheduleKind(task.frequency)),
      sortValue: (task) => scheduleLabel(t, scheduleKind(task.frequency)),
    },
    {
      id: 'next',
      header: t('tasks.list.nextRun'),
      width: 140,
      cell: (task) => (task.nextRun ? time(task.nextRun) : '-'),
      sortValue: (task) => (task.nextRun ? new Date(task.nextRun).getTime() : 0),
    },
    {
      id: 'last',
      header: t('tasks.list.lastRun'),
      width: 140,
      cell: (task) => {
        const started = lastStarted(task);
        return started ? time(started) : '-';
      },
      sortValue: (task) => {
        const started = lastStarted(task);
        return started ? new Date(started).getTime() : 0;
      },
    },
    {
      id: 'result',
      header: t('tasks.list.lastResult'),
      width: 80,
      cell: (task) => (task.lastRun ? resultLabel(t, task.lastRun.state) : '-'),
    },
  ];

  const all = tasks.data ?? [];
  const statusCounts = useMemo(() => countBy(all, statusOf), [all]);
  const categoryCounts = useMemo(() => countBy(all, (task) => categoryOf(task.type)), [all]);
  const column = columns.find((candidate) => candidate.id === sort.column);
  const rows = sortList(
    all.filter(
      (task) =>
        (chosen.status.length === 0 || chosen.status.includes(statusOf(task))) &&
        (chosen.category.length === 0 || chosen.category.includes(categoryOf(task.type))) &&
        matchesQuery(task, query, statusLabel(t, statusOf(task))),
    ),
    column?.sortValue,
    sort.direction,
    collator,
  );
  const sections = [
    {
      id: 'status',
      title: t('tasks.list.status'),
      options: STATUSES.filter((status) => statusCounts.has(status)).map((status) => ({
        value: status,
        label: statusLabel(t, status),
        count: statusCounts.get(status) ?? 0,
      })),
    },
    {
      id: 'category',
      title: t('tasks.list.category'),
      options: [...categoryCounts.entries()]
        .map(([value, count]) => ({ value, label: categoryKeyLabel(t, value as Category), count }))
        .sort((left, right) => collator.compare(left.label, right.label)),
    },
  ];

  const sortBy = (id: string) =>
    setSort(
      sort.column === id
        ? { column: id, direction: sort.direction === 'asc' ? 'desc' : 'asc' }
        : { column: id, direction: 'asc' },
    );

  const rowButtons = (task: Task) => {
    const buttons: Array<{ id: string; label: string; title: string; icon: ReactNode; onClick(): void }> = [];
    if (isActive(task) && canStop) {
      buttons.push({
        id: 'stop',
        label: t('tasks.list.stopTask', { name: task.name }),
        title: t('tasks.actions.stop'),
        icon: <Square size={14} aria-hidden="true" />,
        onClick: () => void attempt(() => actions.stop(task)),
      });
    } else if (canStart && task.enabled && task.known) {
      buttons.push({
        id: 'run',
        label: t('tasks.list.runTask', { name: task.name }),
        title: t('tasks.actions.runNow'),
        icon: <Play size={14} aria-hidden="true" />,
        onClick: () => void attempt(() => actions.run(task)),
      });
    }
    if (canDelete) {
      buttons.push({
        id: 'delete',
        label: t('tasks.list.deleteTask', { name: task.name }),
        title: t('tasks.actions.delete'),
        icon: <Trash2 size={14} aria-hidden="true" />,
        onClick: () => setDeleting(task),
      });
    }
    return buttons;
  };

  const filtered = query !== '' || chosen.status.length > 0 || chosen.category.length > 0;

  return (
    <Page
      className={styles.page}
      title={t('tasks.list.title')}
      description={t('tasks.list.intro')}
      breadcrumbs={[{ label: t('tasks.crumbs.settings'), to: '/settings' }, { label: t('tasks.list.title') }]}
      actions={
        canCreate && (
          <Button asChild size="3" className={`${list.bigButton} ${styles.createButton}`}>
            <Link to={`${LIST_PAGE}/create`}>
              <Plus size={16} aria-hidden="true" />
              {t('tasks.list.create')}
            </Link>
          </Button>
        )
      }
    >
      <div className={list.layout}>
        <FilterPanel
          sections={sections}
          selected={chosen}
          onChange={(section, values) => update(section, values.join(','))}
        />
        <div className={styles.main}>
          <div className={styles.search}>
            <SearchBox
              value={query}
              onChange={(value) => update('q', value)}
              placeholder={t('tasks.list.search')}
            />
          </div>
          {tasks.error ? (
            <ErrorState error={tasks.error} onRetry={() => void tasks.refetch()} />
          ) : (
            <div className={list.tableCard} role="region" aria-label={t('tasks.list.title')}>
              <Table.Root variant="ghost" size="2" className={list.table}>
                <Table.Header>
                  <Table.Row>
                    {columns.map((column) => {
                      const sorted = sort.column === column.id ? sort.direction : undefined;
                      return (
                        <Table.ColumnHeaderCell
                          key={column.id}
                          className={list.th}
                          style={{ width: column.width }}
                          aria-sort={
                            sorted === 'asc' ? 'ascending' : sorted === 'desc' ? 'descending' : undefined
                          }
                        >
                          {column.sortValue ? (
                            <button type="button" className={list.sort} onClick={() => sortBy(column.id)}>
                              <span>{column.header}</span>
                              {sorted === 'asc' && <ArrowUp size={14} aria-hidden="true" />}
                              {sorted === 'desc' && <ArrowDown size={14} aria-hidden="true" />}
                            </button>
                          ) : (
                            <span className={list.thLabel}>{column.header}</span>
                          )}
                        </Table.ColumnHeaderCell>
                      );
                    })}
                    <Table.ColumnHeaderCell className={list.th} style={{ width: 90 }}>
                      <span className={styles.hidden}>{t('tasks.list.actions')}</span>
                    </Table.ColumnHeaderCell>
                    <Table.ColumnHeaderCell className={list.th} style={{ width: 40 }} />
                  </Table.Row>
                </Table.Header>
                <Table.Body>
                  {rows.map((task) => {
                    const open = () => navigate(taskHref(task.id));
                    return (
                      <Table.Row
                        key={task.id}
                        className={`${list.clickable} ${styles.row}`}
                        role="button"
                        tabIndex={0}
                        aria-label={t('tasks.list.open', { name: task.name })}
                        onClick={open}
                        onKeyDown={(event) => {
                          if (
                            event.target === event.currentTarget &&
                            (event.key === 'Enter' || event.key === ' ')
                          ) {
                            event.preventDefault();
                            open();
                          }
                        }}
                      >
                        {columns.map((column) => (
                          <Table.Cell key={column.id} className={list.td}>
                            {column.cell(task)}
                          </Table.Cell>
                        ))}
                        <Table.Cell className={list.td} align="right">
                          <span className={styles.rowActions}>
                            {rowButtons(task).map((button) => (
                              <Tooltip key={button.id} content={button.title}>
                                <button
                                  type="button"
                                  className={styles.rowAction}
                                  data-kind={button.id}
                                  aria-label={button.label}
                                  onClick={(event) => {
                                    event.stopPropagation();
                                    button.onClick();
                                  }}
                                  onKeyDown={(event) => event.stopPropagation()}
                                >
                                  {button.icon}
                                </button>
                              </Tooltip>
                            ))}
                          </span>
                        </Table.Cell>
                        <Table.Cell className={`${list.td} ${styles.chevron}`} align="right">
                          <ChevronRight size={16} aria-hidden="true" />
                        </Table.Cell>
                      </Table.Row>
                    );
                  })}
                </Table.Body>
              </Table.Root>
              {rows.length === 0 && !tasks.isPending && (
                <div className={list.count}>
                  {filtered ? t('tasks.list.noMatches') : t('tasks.list.empty')}
                </div>
              )}
              {tasks.isPending && <div className={list.loading} aria-busy="true" />}
            </div>
          )}
        </div>
      </div>
      <DeleteTaskDialog
        open={deleting !== null}
        onOpenChange={(open) => !open && setDeleting(null)}
        name={deleting?.name ?? ''}
        onDelete={async () => {
          if (deleting) {
            await actions.remove(deleting);
          }
        }}
      />
    </Page>
  );
}
