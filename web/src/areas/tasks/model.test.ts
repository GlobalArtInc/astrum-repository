import { describe, expect, it } from 'vitest';
import {
  type Choices,
  categoryOf,
  cronProblem,
  describeCron,
  emptyTask,
  type FormField,
  fieldOptions,
  fromFrequency,
  matchesQuery,
  missingRequired,
  type ProblemTexts,
  scheduleKind,
  statusOf,
  type Task,
  type Template,
  toFrequency,
  toRequest,
  validateSchedule,
  validateSettings,
} from './model';

function field(change: Partial<FormField>): FormField {
  return {
    id: 'value',
    type: 'string',
    label: 'Value',
    helpText: null,
    required: false,
    initialValue: null,
    minValue: null,
    maxValue: null,
    storeApi: null,
    storeFilters: null,
    attributes: {},
    ...change,
  };
}

const COMPACT: Template = {
  type: 'blobstore.compact',
  name: 'Admin - Compact blob store',
  properties: {},
  formFields: [
    field({
      id: 'blobstoreName',
      type: 'combobox',
      required: true,
      storeApi: 'blobstores',
      attributes: { allowAll: true },
    }),
    field({ id: 'blobsOlderThan', type: 'number', minValue: '0', initialValue: '1' }),
  ],
};

const TEXTS: ProblemTexts = {
  required: (label) => `${label} is required`,
  number: (label) => `${label} must be a number`,
  min: (label, min) => `${label} must be at least ${min}`,
  max: (label, max) => `${label} must be at most ${max}`,
  email: 'email',
  name: 'name',
  days: 'days',
  date: 'date',
  time: 'time',
  cron: 'cron',
  cronParts: 'cronParts',
  cronDay: 'cronDay',
};

describe('schedules', () => {
  it('names the advanced schedule cron and falls back to manual', () => {
    expect(scheduleKind({ schedule: 'advanced' } as never)).toBe('cron');
    expect(scheduleKind({ schedule: 'weekly' } as never)).toBe('weekly');
    expect(scheduleKind({ schedule: 'whenever' } as never)).toBe('manual');
  });

  it('turns date and time in the browser zone into the start date the server stores', () => {
    const values = fromFrequency(null, new Date(2026, 9, 12, 21, 29));
    expect(values).toMatchObject({ schedule: 'manual', date: '2026-10-12', time: '22:00', days: [] });
    const weekly = toFrequency({ ...values, schedule: 'weekly', days: [1, 5] });
    expect(weekly.startDate).toBe(new Date(2026, 9, 12, 22, 0).getTime());
    expect(weekly.recurringDays).toEqual([1, 5]);
    expect(weekly.timeZoneOffset).toMatch(/^(Z|[+-]\d{2}:\d{2})$/);
    expect(fromFrequency(weekly)).toMatchObject({ schedule: 'weekly', date: '2026-10-12', time: '22:00' });
    expect(toFrequency({ ...values, schedule: 'cron', cronExpression: ' 0 0 1 * * ? ' })).toEqual({
      schedule: 'cron',
      startDate: null,
      timeZoneOffset: null,
      recurringDays: null,
      cronExpression: '0 0 1 * * ?',
    });
    expect(toFrequency({ ...values, schedule: 'manual' }).startDate).toBeNull();
  });

  it('checks cron expressions and describes the common ones', () => {
    expect(cronProblem('')).toBe('required');
    expect(cronProblem('0 0 1 * *')).toBe('parts');
    expect(cronProblem('0 0 1 * * *')).toBe('dayOrWeek');
    expect(cronProblem('0 0 1 * * ?')).toBeNull();
    expect(describeCron('0 30 1 * * ?')).toEqual({ key: 'everyDay', time: '01:30' });
    expect(describeCron('0 0 9 ? * MON-FRI')).toEqual({ key: 'weekdays', time: '09:00' });
    expect(describeCron('0 0 */4 * * ?')).toEqual({ key: 'everyHours', count: 4 });
    expect(describeCron('0 0 0 L * ?')).toEqual({ key: 'lastDay', time: '00:00' });
    expect(describeCron('0 0 0 1 1 ?')).toBeNull();
  });

  it('asks for the date, time and days each kind needs', () => {
    const base = fromFrequency(null);
    expect(validateSchedule({ ...base, schedule: 'manual', date: '' }, TEXTS)).toEqual({});
    expect(validateSchedule({ ...base, schedule: 'weekly', days: [] }, TEXTS)).toEqual({
      recurringDays: 'days',
    });
    expect(validateSchedule({ ...base, schedule: 'daily', time: '' }, TEXTS)).toEqual({ startTime: 'time' });
    expect(validateSchedule({ ...base, schedule: 'cron', cronExpression: '* * *' }, TEXTS)).toEqual({
      cronExpression: 'cronParts',
    });
  });
});

describe('task settings', () => {
  it('starts from the template and sends what the public API expects', () => {
    const values = emptyTask(COMPACT);
    expect(values.properties).toEqual({ blobstoreName: '', blobsOlderThan: '1' });
    expect(missingRequired(COMPACT, values)).toBe(true);
    const ready = { ...values, name: ' Compact ', properties: { ...values.properties, blobstoreName: '*' } };
    expect(missingRequired(COMPACT, ready)).toBe(false);
    expect(toRequest(ready, COMPACT.type)).toMatchObject({
      type: 'blobstore.compact',
      name: 'Compact',
      enabled: true,
      alertEmail: null,
      notificationCondition: 'FAILURE',
      properties: { blobstoreName: '*', blobsOlderThan: '1' },
    });
    expect(toRequest(ready)).not.toHaveProperty('type');
  });

  it('checks names, addresses and numbers', () => {
    const values = { ...emptyTask(COMPACT), alertEmail: 'nobody' };
    values.properties.blobsOlderThan = '-1';
    expect(validateSettings(values, COMPACT, TEXTS)).toEqual({
      name: 'name',
      alertEmail: 'email',
      blobstoreName: 'Value is required',
      blobsOlderThan: 'Value must be at least 0',
    });
    values.properties.blobsOlderThan = 'many';
    expect(validateSettings(values, COMPACT, TEXTS).blobsOlderThan).toBe('Value must be a number');
  });

  it('offers repositories and blob stores by format, type and group', () => {
    const choices: Choices = {
      repositories: [
        { name: 'docker-hosted', format: 'docker', type: 'hosted' },
        { name: 'docker-proxy', format: 'docker', type: 'proxy' },
        { name: 'raw', format: 'raw', type: 'hosted' },
      ],
      blobStores: [
        { name: 'default', group: false },
        { name: 'pool', group: true },
      ],
    };
    const repository = field({
      storeApi: 'repositories',
      storeFilters: { format: 'docker', type: 'hosted' },
      attributes: { allowAll: true },
    });
    expect(fieldOptions(repository, choices).map((option) => option.value)).toEqual(['*', 'docker-hosted']);
    const store = field({ storeApi: 'blobstores' });
    expect(fieldOptions(store, choices).map((option) => option.value)).toEqual(['default']);
    expect(
      fieldOptions(field({ storeApi: 'blobstores', attributes: { groups: true } }), choices).map(
        (o) => o.value,
      ),
    ).toEqual(['default', 'pool']);
  });
});

describe('the task list', () => {
  const task = {
    name: 'Nightly compaction',
    type: 'blobstore.compact',
    typeName: 'Admin - Compact blob store',
    enabled: true,
    state: 'waiting',
    lastRun: { message: 'freed 2 blobs' },
  } as unknown as Task;

  it('sorts tasks into statuses and categories', () => {
    expect(statusOf(task)).toBe('waiting');
    expect(statusOf({ ...task, enabled: false })).toBe('disabled');
    expect(statusOf({ ...task, state: 'running' })).toBe('running');
    expect(categoryOf('repository.cleanup')).toBe('cleanup');
    expect(categoryOf('repository.docker.gc')).toBe('docker');
    expect(categoryOf('blobstore.metrics.reconcile')).toBe('repair');
    expect(categoryOf('security.purge-expired')).toBe('admin');
    expect(categoryOf('repository.purge-unused')).toBe('repository');
    expect(categoryOf('something.else')).toBe('other');
  });

  it('searches names, types, statuses and the last message', () => {
    expect(matchesQuery(task, 'nightly compact', 'Waiting')).toBe(true);
    expect(matchesQuery(task, 'waiting', 'Waiting')).toBe(true);
    expect(matchesQuery(task, 'freed', 'Waiting')).toBe(true);
    expect(matchesQuery(task, 'docker', 'Waiting')).toBe(false);
  });
});
