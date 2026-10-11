import TasksStrings from '../../../../constants/pages/settings/system/TasksStrings';
import { TaskStatus } from './types';

/**
 * Display texts of the states and schedules the server reports for a task, shared by the list and the detail page.
 */
const STATUS_LABELS: Record<string, string> = {
  WAITING: TasksStrings.LIST.WAITING,
  RUNNING: TasksStrings.LIST.RUNNING,
  OK: 'OK',
  ERROR: TasksStrings.LIST.ERROR,
  BLOCKED: TasksStrings.LIST.BLOCKED,
  CANCELED: TasksStrings.LIST.CANCELED,
  FAILED: TasksStrings.LIST.FAILED,
  INTERRUPTED: TasksStrings.LIST.INTERRUPTED,
  DISABLED: TasksStrings.LIST.DISABLED,
};

const SCHEDULE_LABELS: Record<string, string> = {
  manual: TasksStrings.LIST.MANUAL,
  once: TasksStrings.LIST.ONCE,
  hourly: TasksStrings.LIST.HOURLY,
  daily: TasksStrings.LIST.DAILY,
  weekly: TasksStrings.LIST.WEEKLY,
  monthly: TasksStrings.LIST.MONTHLY,
  cron: 'Cron',
  advanced: TasksStrings.LIST.CRON,
};

export function getTaskStatusLabel(status: string): string {
  return STATUS_LABELS[status] || status;
}

export function getTaskScheduleLabel(schedule: string | null | undefined): string {
  if (!schedule) {
    return '-';
  }
  return SCHEDULE_LABELS[schedule] || schedule;
}

export function getTaskStatusBadgeColor(status: TaskStatus | 'DISABLED'): 'blue' | 'green' | 'gray' | 'yellow' | 'red' {
  switch (status) {
    case 'RUNNING': return 'blue';
    case 'OK': return 'green';
    case 'BLOCKED': return 'yellow';
    case 'CANCELED':
    case 'FAILED':
    case 'INTERRUPTED':
      return 'red';
    default: return 'gray';
  }
}

/**
 * The state shown for a task: a disabled task does not run, whatever its scheduler state.
 */
export function getEffectiveTaskStatus(task: { enabled: boolean; status: TaskStatus }): TaskStatus | 'DISABLED' {
  return task.enabled ? task.status : 'DISABLED';
}
