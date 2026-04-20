import { INTERNAL, V1 } from '../../api';

export const TASKS = `${V1}/tasks`;
export const UI_TASKS = `${INTERNAL}/ui/tasks`;
export const TYPES = `${UI_TASKS}/types`;
export const CHOICES = `${UI_TASKS}/choices`;
export const LIST_PAGE = '/admin/system/tasks';
export const READ = 'astrum:tasks:read';
export const CREATE = 'astrum:tasks:create';
export const UPDATE = 'astrum:tasks:update';
export const DELETE = 'astrum:tasks:delete';
export const START = 'astrum:tasks:start';
export const STOP = 'astrum:tasks:stop';
export const ALL = '*';
export const LAST_DAY = 999;

export type ScheduleKind = 'manual' | 'once' | 'hourly' | 'daily' | 'weekly' | 'monthly' | 'cron';
export type RunState = 'waiting' | 'running' | 'done' | 'failed' | 'cancelled';
export type TaskState = 'waiting' | 'queued' | 'running';

export interface Frequency {
  schedule: string;
  startDate: number | null;
  timeZoneOffset: string | null;
  recurringDays: number[] | null;
  cronExpression: string | null;
}

export interface Run {
  id: number;
  state: RunState;
  trigger: 'manual' | 'schedule';
  initiator: string;
  node: string;
  message: string;
  counts: Record<string, number>;
  cancelRequested: boolean;
  queuedAt: string;
  startedAt: string | null;
  finishedAt: string | null;
  durationMs: number | null;
}

export interface FormField {
  id: string;
  type: 'string' | 'number' | 'checkbox' | 'combobox';
  label: string;
  helpText: string | null;
  required: boolean;
  initialValue: string | null;
  minValue: string | null;
  maxValue: string | null;
  storeApi: 'repositories' | 'blobstores' | null;
  storeFilters: { format?: string; type?: string } | null;
  attributes: { allowAll?: boolean; groups?: boolean };
}

export interface Template {
  type: string;
  name: string;
  properties: Record<string, string>;
  formFields: FormField[];
  creatable?: boolean;
  singleton?: boolean;
}

export interface Task {
  id: string;
  name: string;
  type: string;
  typeName: string;
  known: boolean;
  enabled: boolean;
  frequency: Frequency;
  properties: Record<string, string>;
  alertEmail: string | null;
  notificationCondition: string;
  nextRun: string | null;
  state: TaskState;
  current: Run | null;
  lastRun: Run | null;
  createdAt: string;
  updatedAt: string;
  template?: Template;
}

export interface Choices {
  repositories: Array<{ name: string; format: string; type: string }>;
  blobStores: Array<{ name: string; group: boolean }>;
}

export interface LogLine {
  line: number;
  at: string;
  level: 'INFO' | 'WARN' | 'ERROR';
  message: string;
}

export interface LogPage {
  run: Run;
  finished: boolean;
  more: boolean;
  lines: LogLine[];
}

export function taskHref(id: string): string {
  return `${LIST_PAGE}/${encodeURIComponent(id)}`;
}

export type Category = 'admin' | 'cleanup' | 'repository' | 'repair' | 'docker' | 'maven' | 'other';

export function categoryOf(type: string): Category {
  if (type === 'repository.cleanup') {
    return 'cleanup';
  }
  if (type.startsWith('repository.docker.')) {
    return 'docker';
  }
  if (type.startsWith('repository.maven.')) {
    return 'maven';
  }
  if (type === 'blobstore.metrics.reconcile') {
    return 'repair';
  }
  if (type.startsWith('blobstore.') || type.startsWith('security.')) {
    return 'admin';
  }
  if (type.startsWith('repository.')) {
    return 'repository';
  }
  return 'other';
}

export function scheduleKind(frequency: Frequency): ScheduleKind {
  const value = frequency.schedule === 'advanced' ? 'cron' : frequency.schedule;
  return (['manual', 'once', 'hourly', 'daily', 'weekly', 'monthly', 'cron'] as const).includes(
    value as ScheduleKind,
  )
    ? (value as ScheduleKind)
    : 'manual';
}

export function isActive(task: Pick<Task, 'state'>): boolean {
  return task.state === 'running' || task.state === 'queued';
}

export interface ScheduleValues {
  schedule: ScheduleKind;
  date: string;
  time: string;
  days: number[];
  cronExpression: string;
}

function pad(value: number): string {
  return String(value).padStart(2, '0');
}

export function localDate(date: Date): string {
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}

export function localTime(date: Date): string {
  return `${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

export function fromFrequency(frequency: Frequency | null, now: Date = new Date()): ScheduleValues {
  const kind = frequency ? scheduleKind(frequency) : 'manual';
  const start = frequency?.startDate ? new Date(frequency.startDate) : null;
  const fallback = new Date(now.getTime() + 60 * 60 * 1000);
  fallback.setMinutes(0, 0, 0);
  const when = start ?? fallback;
  return {
    schedule: kind,
    date: localDate(when),
    time: localTime(when),
    days: frequency?.recurringDays ?? [],
    cronExpression: frequency?.cronExpression ?? '',
  };
}

export function browserOffset(date: Date = new Date()): string {
  const minutes = -date.getTimezoneOffset();
  if (minutes === 0) {
    return 'Z';
  }
  const sign = minutes < 0 ? '-' : '+';
  const absolute = Math.abs(minutes);
  return `${sign}${pad(Math.floor(absolute / 60))}:${pad(absolute % 60)}`;
}

export function toFrequency(values: ScheduleValues): Frequency {
  const base: Frequency = {
    schedule: values.schedule,
    startDate: null,
    timeZoneOffset: null,
    recurringDays: null,
    cronExpression: null,
  };
  switch (values.schedule) {
    case 'manual':
      return base;
    case 'cron':
      return { ...base, cronExpression: values.cronExpression.trim(), timeZoneOffset: null };
    default: {
      const start = new Date(`${values.date}T${values.time || '00:00'}:00`);
      return {
        ...base,
        startDate: Number.isNaN(start.getTime()) ? null : start.getTime(),
        timeZoneOffset: browserOffset(start),
        recurringDays:
          values.schedule === 'weekly' || values.schedule === 'monthly' ? [...values.days] : null,
      };
    }
  }
}

const CRON_FIELDS = 6;

export function cronProblem(expression: string): 'required' | 'parts' | 'dayOrWeek' | null {
  const parts = expression.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) {
    return 'required';
  }
  if (parts.length < CRON_FIELDS || parts.length > CRON_FIELDS + 1) {
    return 'parts';
  }
  const dayOfMonth = parts[3];
  const dayOfWeek = parts[5];
  if ((dayOfMonth === '?') === (dayOfWeek === '?')) {
    return 'dayOrWeek';
  }
  return null;
}

export type CronText =
  | { key: 'everyDay'; time: string }
  | { key: 'weekdays'; time: string }
  | { key: 'everyHour' }
  | { key: 'everyHours'; count: number }
  | { key: 'everyMinutes'; count: number }
  | { key: 'monthDay'; day: number; time: string }
  | { key: 'lastDay'; time: string }
  | null;

function number(text: string | undefined): number | null {
  return text !== undefined && /^\d+$/.test(text) ? Number(text) : null;
}

export function describeCron(expression: string): CronText {
  const parts = expression.trim().split(/\s+/);
  if (cronProblem(expression) !== null) {
    return null;
  }
  const [seconds, minutes, hours, dayOfMonth, month, dayOfWeek] = parts;
  if (seconds !== '0' || month !== '*') {
    return null;
  }
  const minute = number(minutes);
  const hour = number(hours);
  const time = minute !== null && hour !== null ? `${pad(hour)}:${pad(minute)}` : null;
  if (time && dayOfWeek === '?' && dayOfMonth === '*') {
    return { key: 'everyDay', time };
  }
  if (time && dayOfMonth === '?' && dayOfWeek?.toUpperCase() === 'MON-FRI') {
    return { key: 'weekdays', time };
  }
  if (time && dayOfWeek === '?' && dayOfMonth === 'L') {
    return { key: 'lastDay', time };
  }
  const day = number(dayOfMonth);
  if (time && dayOfWeek === '?' && day !== null) {
    return { key: 'monthDay', day, time };
  }
  if (minute === 0 && dayOfMonth === '*' && dayOfWeek === '?') {
    if (hours === '*') {
      return { key: 'everyHour' };
    }
    const every = hours?.match(/^(?:\*|0)\/(\d+)$/);
    if (every) {
      return { key: 'everyHours', count: Number(every[1]) };
    }
  }
  const everyMinutes = minutes?.match(/^(?:\*|0)\/(\d+)$/);
  if (everyMinutes && hours === '*' && dayOfMonth === '*' && dayOfWeek === '?') {
    return { key: 'everyMinutes', count: Number(everyMinutes[1]) };
  }
  return null;
}

export interface TaskValues {
  enabled: boolean;
  name: string;
  alertEmail: string;
  notificationCondition: string;
  properties: Record<string, string>;
  schedule: ScheduleValues;
}

export function initialProperties(
  template: Template,
  stored?: Record<string, string>,
): Record<string, string> {
  const values: Record<string, string> = {};
  for (const field of template.formFields) {
    values[field.id] = stored?.[field.id] ?? field.initialValue ?? '';
  }
  return values;
}

export function fromTask(task: Task, template: Template): TaskValues {
  return {
    enabled: task.enabled,
    name: task.name,
    alertEmail: task.alertEmail ?? '',
    notificationCondition: task.notificationCondition,
    properties: initialProperties(template, task.properties),
    schedule: fromFrequency(task.frequency),
  };
}

export function emptyTask(template: Template): TaskValues {
  return {
    enabled: true,
    name: '',
    alertEmail: '',
    notificationCondition: 'FAILURE',
    properties: initialProperties(template),
    schedule: fromFrequency(null),
  };
}

export function toRequest(values: TaskValues, type?: string) {
  return {
    ...(type ? { type } : {}),
    name: values.name.trim(),
    enabled: values.enabled,
    alertEmail: values.alertEmail.trim() === '' ? null : values.alertEmail.trim(),
    notificationCondition: values.notificationCondition,
    frequency: toFrequency(values.schedule),
    properties: values.properties,
  };
}

const EMAIL = /^[^\s@,]+@[^\s@,]+\.[^\s@,]+$/;

export type Problems = Record<string, string>;

export interface ProblemTexts {
  required(label: string): string;
  number(label: string): string;
  min(label: string, min: number): string;
  max(label: string, max: number): string;
  email: string;
  name: string;
  days: string;
  date: string;
  time: string;
  cron: string;
  cronParts: string;
  cronDay: string;
}

export function validateSettings(
  values: TaskValues,
  template: Template,
  texts: ProblemTexts,
  labelOf: (field: FormField) => string = (field) => field.label,
): Problems {
  const problems: Problems = {};
  if (values.name.trim() === '') {
    problems.name = texts.name;
  }
  if (values.alertEmail.trim() !== '' && !EMAIL.test(values.alertEmail.trim())) {
    problems.alertEmail = texts.email;
  }
  for (const field of template.formFields) {
    const value = (values.properties[field.id] ?? '').trim();
    if (value === '') {
      if (field.required && field.type !== 'checkbox') {
        problems[field.id] = texts.required(labelOf(field));
      }
      continue;
    }
    if (field.type === 'number') {
      if (!/^-?\d+$/.test(value)) {
        problems[field.id] = texts.number(labelOf(field));
      } else if (field.minValue !== null && Number(value) < Number(field.minValue)) {
        problems[field.id] = texts.min(labelOf(field), Number(field.minValue));
      } else if (field.maxValue !== null && Number(value) > Number(field.maxValue)) {
        problems[field.id] = texts.max(labelOf(field), Number(field.maxValue));
      }
    }
  }
  return problems;
}

export function missingRequired(template: Template, values: TaskValues): boolean {
  return template.formFields.some(
    (field) =>
      field.required && field.type !== 'checkbox' && (values.properties[field.id] ?? '').trim() === '',
  );
}

export function validateSchedule(values: ScheduleValues, texts: ProblemTexts): Problems {
  const problems: Problems = {};
  switch (values.schedule) {
    case 'manual':
      return problems;
    case 'cron': {
      const problem = cronProblem(values.cronExpression);
      if (problem) {
        problems.cronExpression =
          problem === 'required' ? texts.cron : problem === 'parts' ? texts.cronParts : texts.cronDay;
      }
      return problems;
    }
    default:
      if (values.date === '') {
        problems.startDate = texts.date;
      }
      if (values.time === '') {
        problems.startTime = texts.time;
      }
      if ((values.schedule === 'weekly' || values.schedule === 'monthly') && values.days.length === 0) {
        problems.recurringDays = texts.days;
      }
      return problems;
  }
}

export function fieldOptions(
  field: FormField,
  choices: Choices | undefined,
): Array<{ value: string; label: string; all?: boolean }> {
  const options: Array<{ value: string; label: string; all?: boolean }> = [];
  if (field.attributes.allowAll) {
    options.push({ value: ALL, label: ALL, all: true });
  }
  if (!choices) {
    return options;
  }
  if (field.storeApi === 'repositories') {
    const formats = field.storeFilters?.format?.split(',').filter(Boolean) ?? [];
    const types = field.storeFilters?.type?.split(',').filter(Boolean) ?? [];
    for (const repository of choices.repositories) {
      if (
        (formats.length === 0 || formats.includes(repository.format)) &&
        (types.length === 0 || types.includes(repository.type))
      ) {
        options.push({ value: repository.name, label: repository.name });
      }
    }
  }
  if (field.storeApi === 'blobstores') {
    for (const store of choices.blobStores) {
      if (field.attributes.groups || !store.group) {
        options.push({ value: store.name, label: store.name });
      }
    }
  }
  return options;
}

export function matchesQuery(task: Task, query: string, statusLabel: string): boolean {
  const words = query.trim().toLowerCase().split(/\s+/).filter(Boolean);
  const text =
    `${task.name} ${task.typeName} ${task.type} ${statusLabel} ${task.lastRun?.message ?? ''}`.toLowerCase();
  return words.every((word) => text.includes(word));
}

export type StatusFilter = 'disabled' | 'waiting' | 'queued' | 'running';

export function statusOf(task: Task): StatusFilter {
  if (task.state === 'running') {
    return 'running';
  }
  if (task.state === 'queued') {
    return 'queued';
  }
  return task.enabled ? 'waiting' : 'disabled';
}

export function countBy<T, K extends string>(items: readonly T[], key: (item: T) => K): Map<K, number> {
  const counts = new Map<K, number>();
  for (const item of items) {
    const value = key(item);
    counts.set(value, (counts.get(value) ?? 0) + 1);
  }
  return counts;
}
