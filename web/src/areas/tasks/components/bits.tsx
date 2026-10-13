import { Badge } from '@radix-ui/themes';
import { Ban, CircleCheck, CircleX, Clock, Hourglass, RefreshCw } from 'lucide-react';
import { type Translate, useI18n } from '../../../i18n';
import { type Category, categoryOf, type RunState, type ScheduleKind, type StatusFilter } from '../model';
import styles from '../Tasks.module.css';

export function useLocalTime(): (value: string | number | null | undefined) => string {
  const { language } = useI18n();
  return (value) => (value === null || value === undefined ? '' : new Date(value).toLocaleString(language));
}

const CATEGORY_LABELS: Record<Category, string> = {
  admin: 'tasks.categories.admin',
  cleanup: 'tasks.categories.cleanup',
  repository: 'tasks.categories.repository',
  repair: 'tasks.categories.repair',
  docker: 'tasks.categories.docker',
  maven: 'tasks.categories.maven',
  other: 'tasks.categories.other',
};

export function categoryLabel(t: Translate, type: string): string {
  return t(CATEGORY_LABELS[categoryOf(type)]);
}

export function categoryKeyLabel(t: Translate, category: Category): string {
  return t(CATEGORY_LABELS[category]);
}

const DESCRIPTIONS: Record<string, string> = {
  'repository.cleanup': 'tasks.types.cleanup',
  'blobstore.compact': 'tasks.types.compact',
  'blobstore.metrics.reconcile': 'tasks.types.reconcile',
  'repository.purge-unused': 'tasks.types.purgeUnused',
  'repository.docker.gc': 'tasks.types.dockerGc',
  'security.purge-expired': 'tasks.types.purgeExpired',
};

export function typeDescription(t: Translate, type: string): string {
  return t(DESCRIPTIONS[type] ?? 'tasks.types.other');
}

const SCHEDULE_LABELS: Record<ScheduleKind, string> = {
  manual: 'tasks.schedules.manual',
  once: 'tasks.schedules.once',
  hourly: 'tasks.schedules.hourly',
  daily: 'tasks.schedules.daily',
  weekly: 'tasks.schedules.weekly',
  monthly: 'tasks.schedules.monthly',
  cron: 'tasks.schedules.cron',
};

export function scheduleLabel(t: Translate, kind: ScheduleKind): string {
  return t(SCHEDULE_LABELS[kind]);
}

const OPTION_LABELS: Record<ScheduleKind, string> = {
  manual: 'tasks.schedules.manual',
  once: 'tasks.schedules.once',
  hourly: 'tasks.schedules.hourly',
  daily: 'tasks.schedules.daily',
  weekly: 'tasks.schedules.weekly',
  monthly: 'tasks.schedules.monthly',
  cron: 'tasks.schedules.advanced',
};

export function scheduleOptions(t: Translate): Array<{ value: ScheduleKind; label: string }> {
  return (Object.keys(OPTION_LABELS) as ScheduleKind[]).map((value) => ({
    value,
    label: t(OPTION_LABELS[value]),
  }));
}

const STATUS_LABELS: Record<StatusFilter, string> = {
  disabled: 'tasks.status.disabled',
  waiting: 'tasks.status.waiting',
  queued: 'tasks.status.queued',
  running: 'tasks.status.running',
};

export function statusLabel(t: Translate, status: StatusFilter): string {
  return t(STATUS_LABELS[status]);
}

export function StateBadge({ status, label }: { status: StatusFilter; label: string }) {
  const icon =
    status === 'running' ? (
      <RefreshCw size={14} aria-hidden="true" className={styles.spinning} />
    ) : status === 'queued' ? (
      <Hourglass size={14} aria-hidden="true" />
    ) : status === 'disabled' ? (
      <Ban size={14} aria-hidden="true" />
    ) : (
      <Clock size={14} aria-hidden="true" />
    );
  return (
    <span className={styles.status} data-status={status}>
      {icon}
      <Badge size="1" color={status === 'running' ? undefined : 'gray'} variant="soft">
        {label}
      </Badge>
    </span>
  );
}

export function TitleStateBadge({ status, label }: { status: StatusFilter; label: string }) {
  const Icon =
    status === 'running' ? RefreshCw : status === 'queued' ? Hourglass : status === 'disabled' ? Ban : Clock;
  return (
    <Badge
      size="1"
      color={status === 'running' ? undefined : 'gray'}
      variant="soft"
      className={styles.titleBadge}
    >
      <Icon size={12} aria-hidden="true" className={status === 'running' ? styles.spinning : undefined} />
      {label}
    </Badge>
  );
}

const RESULT_LABELS: Record<RunState, string> = {
  waiting: 'tasks.results.waiting',
  running: 'tasks.results.running',
  done: 'tasks.results.ok',
  failed: 'tasks.results.failed',
  cancelled: 'tasks.results.cancelled',
};

export function resultLabel(t: Translate, state: RunState): string {
  return t(RESULT_LABELS[state]);
}

export function ResultBadge({ state, label }: { state: RunState; label: string }) {
  if (state === 'done') {
    return (
      <span className={styles.status}>
        <CircleCheck size={16} color="var(--green-11)" aria-hidden="true" />
        <Badge color="green" variant="soft">
          {label}
        </Badge>
      </span>
    );
  }
  if (state === 'failed') {
    return (
      <span className={styles.status}>
        <CircleX size={16} color="var(--red-11)" aria-hidden="true" />
        <Badge color="red" variant="soft">
          {label}
        </Badge>
      </span>
    );
  }
  return (
    <Badge color={state === 'cancelled' ? 'amber' : 'gray'} variant="soft">
      {label}
    </Badge>
  );
}

export function duration(t: Translate, milliseconds: number | null): string {
  if (milliseconds === null) {
    return '';
  }
  if (milliseconds < 1000) {
    return t('tasks.duration.milliseconds', { count: milliseconds });
  }
  const seconds = Math.round(milliseconds / 1000);
  if (seconds < 60) {
    return t('tasks.duration.seconds', { count: seconds });
  }
  return t('tasks.duration.minutes', { minutes: Math.floor(seconds / 60), seconds: seconds % 60 });
}
