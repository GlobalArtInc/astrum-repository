import { Callout, Checkbox, TextField as RadixTextField, Text } from '@radix-ui/themes';
import { CalendarDays, Clock, Info } from 'lucide-react';
import { useId } from 'react';
import { useI18n, useT } from '../../../i18n';
import { FieldShell, SelectField } from '../../../ui';
import { describeCron, LAST_DAY, type Problems, type ScheduleKind, type ScheduleValues } from '../model';
import styles from '../Tasks.module.css';
import { scheduleOptions } from './bits';

const WEEK: Array<{ value: number; label: string }> = [
  { value: 1, label: 'tasks.days.sunday' },
  { value: 2, label: 'tasks.days.monday' },
  { value: 3, label: 'tasks.days.tuesday' },
  { value: 4, label: 'tasks.days.wednesday' },
  { value: 5, label: 'tasks.days.thursday' },
  { value: 6, label: 'tasks.days.friday' },
  { value: 7, label: 'tasks.days.saturday' },
];

const TITLES: Partial<Record<ScheduleKind, string>> = {
  once: 'tasks.schedule.runOnce',
  hourly: 'tasks.schedule.runHourly',
  daily: 'tasks.schedule.runDaily',
};

const EXAMPLES = [
  { expression: '0 0 3 * * ?', label: 'tasks.schedule.exampleDaily' },
  { expression: '0 0 */4 * * ?', label: 'tasks.schedule.exampleHours' },
  { expression: '0 0 9 ? * MON-FRI', label: 'tasks.schedule.exampleWeekdays' },
  { expression: '0 30 8 1 * ?', label: 'tasks.schedule.exampleMonthly' },
];

function DateTimeInput({
  label,
  type,
  value,
  onChange,
  error,
  note,
  disabled,
}: {
  label: string;
  type: 'date' | 'time';
  value: string;
  onChange(value: string): void;
  error?: string;
  note?: string;
  disabled?: boolean;
}) {
  const id = useId();
  const Icon = type === 'date' ? CalendarDays : Clock;
  return (
    <div className={styles.dateTime}>
      <label htmlFor={id} className={styles.dateTimeLabel}>
        <Icon size={14} aria-hidden="true" />
        {label}
      </label>
      <RadixTextField.Root
        id={id}
        type={type}
        value={value}
        disabled={disabled}
        aria-invalid={error ? true : undefined}
        aria-describedby={error ? `${id}-error` : undefined}
        onChange={(event) => onChange(event.target.value)}
      />
      {note && (
        <Text size="1" color="gray">
          {note}
        </Text>
      )}
      {error && (
        <Text id={`${id}-error`} size="1" color="red" role="alert">
          {error}
        </Text>
      )}
    </div>
  );
}

function clock(language: string, time: string): string {
  const [hours, minutes] = time.split(':').map(Number);
  return new Date(2000, 0, 1, hours, minutes).toLocaleTimeString(language, {
    hour: 'numeric',
    minute: '2-digit',
  });
}

export function useCronText(): (expression: string) => string | null {
  const t = useT();
  const { language } = useI18n();
  return (expression) => {
    const described = describeCron(expression);
    if (!described) {
      return null;
    }
    switch (described.key) {
      case 'everyDay':
        return t('tasks.schedule.cronEveryDay', { time: clock(language, described.time) });
      case 'weekdays':
        return t('tasks.schedule.cronWeekdays', { time: clock(language, described.time) });
      case 'everyHour':
        return t('tasks.schedule.cronEveryHour');
      case 'everyHours':
        return t('tasks.schedule.cronEveryHours', { count: described.count });
      case 'everyMinutes':
        return t('tasks.schedule.cronEveryMinutes', { count: described.count });
      case 'monthDay':
        return t('tasks.schedule.cronMonthDay', {
          day: described.day,
          time: clock(language, described.time),
        });
      case 'lastDay':
        return t('tasks.schedule.cronLastDay', { time: clock(language, described.time) });
    }
  };
}

export function ScheduleFields({
  values,
  onChange,
  errors,
  disabled,
}: {
  values: ScheduleValues;
  onChange(values: ScheduleValues): void;
  errors: Problems;
  disabled?: boolean;
}) {
  const t = useT();
  const cronText = useCronText();
  const cronId = useId();
  const set = <K extends keyof ScheduleValues>(name: K, value: ScheduleValues[K]) =>
    onChange({ ...values, [name]: value });
  const toggleDay = (day: number) =>
    set(
      'days',
      values.days.includes(day)
        ? values.days.filter((known) => known !== day)
        : [...values.days, day].sort((left, right) => left - right),
    );
  const zone = Intl.DateTimeFormat().resolvedOptions().timeZone;
  const kind = values.schedule;
  const timed = kind !== 'manual' && kind !== 'cron';
  const cronValid = kind === 'cron' && values.cronExpression.trim() !== '' && !errors.cronExpression;
  const described = cronValid ? cronText(values.cronExpression) : null;
  return (
    <div className={styles.section}>
      <SelectField
        label={t('tasks.schedule.schedule')}
        hint={t('tasks.schedule.scheduleHint')}
        required
        value={kind}
        disabled={disabled}
        onChange={(value) => {
          const next = value as ScheduleKind;
          onChange({ ...values, schedule: next, days: next === kind ? values.days : [] });
        }}
        options={scheduleOptions(t)}
      />
      {kind === 'manual' && (
        <Callout.Root color="gray" variant="surface">
          <Callout.Icon>
            <Info size={16} />
          </Callout.Icon>
          <Callout.Text>{t('tasks.schedule.manualOnly')}</Callout.Text>
        </Callout.Root>
      )}
      {timed && (
        <div className={styles.card}>
          {TITLES[kind] && (
            <Text as="p" size="2" weight="medium">
              {t(TITLES[kind])}
            </Text>
          )}
          <div className={styles.inline}>
            <DateTimeInput
              label={kind === 'once' ? t('tasks.schedule.date') : t('tasks.schedule.startDate')}
              type="date"
              value={values.date}
              onChange={(value) => set('date', value)}
              error={errors.startDate}
              disabled={disabled}
            />
            <DateTimeInput
              label={kind === 'hourly' ? t('tasks.schedule.startTime', { zone }) : t('tasks.schedule.time')}
              type="time"
              value={values.time}
              onChange={(value) => set('time', value)}
              error={errors.startTime}
              note={kind === 'hourly' ? undefined : t('tasks.schedule.localZone', { zone })}
              disabled={disabled}
            />
          </div>
          {kind === 'weekly' && (
            <fieldset
              className={styles.daysField}
              aria-describedby={errors.recurringDays ? 'week-error' : undefined}
            >
              <legend className={styles.daysTitle}>{t('tasks.schedule.days')}</legend>
              <div className={styles.days}>
                {WEEK.map((day) => (
                  <Text as="label" size="2" key={day.value} className={styles.dayChip}>
                    <Checkbox
                      checked={values.days.includes(day.value)}
                      disabled={disabled}
                      onCheckedChange={() => toggleDay(day.value)}
                    />
                    {t(day.label)}
                  </Text>
                ))}
              </div>
              {errors.recurringDays && (
                <Text id="week-error" size="1" color="red" role="alert">
                  {errors.recurringDays}
                </Text>
              )}
            </fieldset>
          )}
          {kind === 'monthly' && (
            <fieldset className={styles.daysField}>
              <legend className={styles.daysTitle}>{t('tasks.schedule.monthDays')}</legend>
              <div className={styles.monthDays}>
                {[...Array.from({ length: 31 }, (_, index) => index + 1), LAST_DAY].map((day) => (
                  <button
                    type="button"
                    key={day}
                    className={styles.monthDay}
                    aria-pressed={values.days.includes(day)}
                    disabled={disabled}
                    onClick={() => toggleDay(day)}
                  >
                    {day === LAST_DAY ? t('tasks.schedule.lastDay') : day}
                  </button>
                ))}
              </div>
              {errors.recurringDays && (
                <Text size="1" color="red" role="alert">
                  {errors.recurringDays}
                </Text>
              )}
            </fieldset>
          )}
        </div>
      )}
      {kind === 'cron' && (
        <div className={styles.card}>
          <FieldShell
            id={cronId}
            label={t('tasks.schedule.cronExpression')}
            hint={t('tasks.schedule.cronHint')}
            error={errors.cronExpression}
            required
          >
            <RadixTextField.Root
              id={cronId}
              value={values.cronExpression}
              disabled={disabled}
              style={{ fontFamily: 'var(--code-font-family)' }}
              aria-invalid={errors.cronExpression ? true : undefined}
              onChange={(event) => set('cronExpression', event.target.value)}
            />
          </FieldShell>
          {cronValid && (
            <Text size="1" color="green">
              {described ?? t('tasks.schedule.cronValid')}
            </Text>
          )}
          <div className={styles.examples}>
            <Text size="1" weight="bold">
              {t('tasks.schedule.examples')}
            </Text>
            {EXAMPLES.map((example) => (
              <Text size="1" key={example.expression} color="gray">
                <code>{example.expression}</code> - {t(example.label)}
              </Text>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
