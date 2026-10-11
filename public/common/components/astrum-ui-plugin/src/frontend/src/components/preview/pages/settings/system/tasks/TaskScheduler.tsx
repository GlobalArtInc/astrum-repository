

import React, { useCallback, useMemo } from 'react';
import { Box, Flex, Text, Grid, Checkbox, TextField } from '@radix-ui/themes';
import { Calendar, Clock, Info } from 'lucide-react';

import {
  SettingsSelect,
  SettingsTextInput,
} from '../../../../shared/form';

import {
  ScheduleType,
  TaskSchedulerProps,
  SCHEDULE_OPTIONS,
  WEEKDAYS,
  MONTH_DAYS,
  DEFAULT_SCHEDULE_DATA,
  getTimezoneOffset,
  validateCronExpression,
} from './types';
import plural from '../../../../../../interface/plural';
import TasksStrings from '../../../../constants/pages/settings/system/TasksStrings';
import { formatClockTime } from '../../../../shared/utils/formatLocale';

const DAY_NAMES: Record<string, string> = {
  SUN: TasksStrings.SCHEDULER.SUNDAY, MON: TasksStrings.SCHEDULER.MONDAY, TUE: TasksStrings.SCHEDULER.TUESDAY, WED: TasksStrings.SCHEDULER.WEDNESDAY,
  THU: TasksStrings.SCHEDULER.THURSDAY, FRI: TasksStrings.SCHEDULER.FRIDAY, SAT: TasksStrings.SCHEDULER.SATURDAY,
  '1': TasksStrings.SCHEDULER.SUNDAY, '2': TasksStrings.SCHEDULER.MONDAY, '3': TasksStrings.SCHEDULER.TUESDAY, '4': TasksStrings.SCHEDULER.WEDNESDAY,
  '5': TasksStrings.SCHEDULER.THURSDAY, '6': TasksStrings.SCHEDULER.FRIDAY, '7': TasksStrings.SCHEDULER.SATURDAY,
};

export function describeCron(expr: string): string {
  const parts = expr.trim().split(/\s+/);
  if (parts.length < 6) return '';

  const [_sec, min, hour, dom, , dow] = parts;

  const formatTime = (h: string, m: string) => formatClockTime(parseInt(h, 10), parseInt(m, 10));

  try {
    if (min.startsWith('*/')) {
      return plural(TasksStrings.SCHEDULER.EVERY_MINUTES, Number(min.slice(2)), min.slice(2));
    }
    if (hour === '*') {
      return TasksStrings.SCHEDULER.EVERY_HOUR;
    }
    if (hour.startsWith('*/')) {
      return plural(TasksStrings.SCHEDULER.EVERY_HOURS, Number(hour.slice(2)), hour.slice(2));
    }

    const isEveryDom = dom === '*' || dom === '?';
    const isEveryDow = dow === '*' || dow === '?';

    if (dow === 'MON-FRI' && isEveryDom) {
      return TasksStrings.SCHEDULER.WEEKDAYS_AT(formatTime(hour, min));
    }
    if (dow === 'SAT,SUN' && isEveryDom) {
      return TasksStrings.SCHEDULER.WEEKENDS_AT(formatTime(hour, min));
    }

    if (!isEveryDow && isEveryDom) {
      const days = dow.split(',').map((d) => DAY_NAMES[d] || d).join(', ');
      return TasksStrings.SCHEDULER.AT(days, formatTime(hour, min));
    }

    if (!isEveryDom && isEveryDow) {
      if (dom === 'L') return TasksStrings.SCHEDULER.LAST_DAY_MONTH(formatTime(hour, min));
      return TasksStrings.SCHEDULER.DAY_EACH_MONTH(dom, formatTime(hour, min));
    }

    if (isEveryDom && isEveryDow) {
      return TasksStrings.SCHEDULER.EVERY_DAY(formatTime(hour, min));
    }

    return TasksStrings.SCHEDULER.RUNS_AT(formatTime(hour, min));
  } catch {
    return '';
  }
}

import './TaskScheduler.scss';

/**
 * Get local timezone name for display
 */
const getTimezoneName = (): string => {
  try {
    return Intl.DateTimeFormat().resolvedOptions().timeZone;
  } catch {
    return TasksStrings.SCHEDULER.LOCAL_TIMEZONE;
  }
};

/**
 * TaskScheduler - Schedule picker component with support for all schedule types
 * 
 * Schedule types:
 * - Manual: No automatic scheduling
 * - Once: Run once at a specific date/time
 * - Hourly: Run every hour starting at a specific time
 * - Daily: Run daily at a specific time
 * - Weekly: Run on specific days of the week at a specific time
 * - Monthly: Run on specific days of the month at a specific time
 * - Advanced: Custom cron expression
 */
export function TaskScheduler({
  value = DEFAULT_SCHEDULE_DATA,
  onChange,
  errors = {},
  disabled = false,
  allowedSchedules,
}: TaskSchedulerProps) {
  const timezone = useMemo(() => getTimezoneName(), []);

  // Handle schedule type change.
  const handleScheduleChange = useCallback((newSchedule: string) => {
    const scheduleType = newSchedule as ScheduleType;
    const isTimeBased = scheduleType !== 'manual' && scheduleType !== 'advanced';
    const carriedStartDate = value.startDate ? new Date(value.startDate) : null;
    onChange({
      ...DEFAULT_SCHEDULE_DATA,
      schedule: scheduleType,
      timeZoneOffset: getTimezoneOffset(),
      startDate: isTimeBased ? (carriedStartDate || new Date()) : null,
      startTime: isTimeBased ? (value.startTime || '00:00') : undefined,
    });
  }, [onChange, value.startDate, value.startTime]);

  // Handle date change
  const handleDateChange = useCallback((dateStr: string) => {
    // Parse as local midnight, not UTC midnight. new Date("YYYY-MM-DD") treats the
    // string as UTC which shifts the calendar date in negative-offset timezones
    // (e.g. CDT UTC-5: "2026-06-30" → June 29 local). The three-argument constructor
    // always produces local midnight regardless of timezone.
    let date: Date | null = null;
    if (dateStr) {
      const [y, m, d] = dateStr.split('-').map(Number);
      date = new Date(y, m - 1, d);
    }
    onChange({
      ...value,
      startDate: date,
      timeZoneOffset: getTimezoneOffset(),
    });
  }, [onChange, value]);

  // Handle time change
  const handleTimeChange = useCallback((time: string) => {
    onChange({
      ...value,
      startTime: time,
      timeZoneOffset: getTimezoneOffset(),
    });
  }, [onChange, value]);

  // Handle recurring day toggle (for weekly/monthly)
  const handleDayToggle = useCallback((day: number) => {
    const currentDays = value.recurringDays || [];
    const newDays = currentDays.includes(day)
      ? currentDays.filter(d => d !== day)
      : [...currentDays, day].sort((a, b) => a - b);
    
    onChange({
      ...value,
      recurringDays: newDays,
      timeZoneOffset: getTimezoneOffset(),
    });
  }, [onChange, value]);

  // Handle cron expression change
  const handleCronChange = useCallback((cron: string) => {
    onChange({
      ...value,
      cronExpression: cron,
      timeZoneOffset: getTimezoneOffset(),
    });
  }, [onChange, value]);

  // Format date for input
  const dateValue = useMemo(() => {
    if (!value.startDate) return '';
    const d = new Date(value.startDate);
    // Use local date getters, not toISOString (which returns UTC). In UTC+ timezones
    // a local midnight Date has a UTC date one day earlier, so toISOString().split('T')[0]
    // would display the wrong calendar day.
    const yyyy = d.getFullYear();
    const mm = String(d.getMonth() + 1).padStart(2, '0');
    const dd = String(d.getDate()).padStart(2, '0');
    return `${yyyy}-${mm}-${dd}`;
  }, [value.startDate]);

  // Render schedule-specific fields based on type
  const renderScheduleFields = () => {
    switch (value.schedule) {
      case 'manual':
        return (
          <Box className="task-scheduler__info">
            <Info size={16} />
            <Text size="2">{TasksStrings.SCHEDULER.TASK_ONLY_RUN_WHEN_MANUALLY}</Text>
          </Box>
        );

      case 'once':
        return (
          <Flex direction="column" gap="3">
            <Text size="2" weight="medium" className="task-scheduler__section-title">
              {TasksStrings.SCHEDULER.RUN_ONCE}
            </Text>
            <Flex gap="3" wrap="wrap">
              <Box className="task-scheduler__field task-scheduler__field--date">
                <label htmlFor="schedule-date" className="task-scheduler__label">
                  <Calendar size={14} />
                  {TasksStrings.SCHEDULER.DATE}
                </label>
                <TextField.Root
                  id="schedule-date"
                  type="date"
                  value={dateValue}
                  onChange={(e) => handleDateChange(e.target.value)}
                  disabled={disabled}
                  aria-describedby={errors.startDate ? 'schedule-date-error' : undefined}
                  aria-invalid={errors.startDate ? true : undefined}
                />
                {errors.startDate && (
                  <Text id="schedule-date-error" size="1" className="task-scheduler__error">{errors.startDate}</Text>
                )}
              </Box>
              <Box className="task-scheduler__field task-scheduler__field--time">
                <label htmlFor="schedule-time" className="task-scheduler__label">
                  <Clock size={14} />
                  {TasksStrings.SCHEDULER.TIME}
                </label>
                <TextField.Root
                  id="schedule-time"
                  type="time"
                  value={value.startTime || '00:00'}
                  onChange={(e) => handleTimeChange(e.target.value)}
                  disabled={disabled}
                  autoComplete="off"
                  aria-describedby={errors.startTime ? 'schedule-time-error' : undefined}
                  aria-invalid={errors.startTime ? true : undefined}
                />
                <Text size="1" color="gray" mt="1">{TasksStrings.SCHEDULER.TIMES_SHOWN_LOCAL_TIMEZONE(timezone)}</Text>
                {errors.startTime && (
                  <Text id="schedule-time-error" size="1" className="task-scheduler__error">{errors.startTime}</Text>
                )}
              </Box>
            </Flex>
          </Flex>
        );

      case 'hourly':
        return (
          <Flex direction="column" gap="3">
            <Text size="2" weight="medium" className="task-scheduler__section-title">
              {TasksStrings.SCHEDULER.RUN_EVERY_HOUR_STARTING}
            </Text>
            <Flex gap="3" wrap="wrap">
              <Box className="task-scheduler__field task-scheduler__field--date">
                <label htmlFor="schedule-date" className="task-scheduler__label">
                  <Calendar size={14} />
                  {TasksStrings.SCHEDULER.START_DATE}
                </label>
                <TextField.Root
                  id="schedule-date"
                  type="date"
                  value={dateValue}
                  onChange={(e) => handleDateChange(e.target.value)}
                  disabled={disabled}
                  aria-describedby={errors.startDate ? 'schedule-date-error' : undefined}
                  aria-invalid={errors.startDate ? true : undefined}
                />
                {errors.startDate && (
                  <Text id="schedule-date-error" size="1" className="task-scheduler__error">{errors.startDate}</Text>
                )}
              </Box>
              <Box className="task-scheduler__field task-scheduler__field--time">
                <label htmlFor="schedule-time" className="task-scheduler__label">
                  <Clock size={14} />
                  {TasksStrings.SCHEDULER.START_TIME(timezone)}
                </label>
                <TextField.Root
                  id="schedule-time"
                  type="time"
                  value={value.startTime || '00:00'}
                  onChange={(e) => handleTimeChange(e.target.value)}
                  disabled={disabled}
                  aria-describedby={errors.startTime ? 'schedule-time-error' : undefined}
                  aria-invalid={errors.startTime ? true : undefined}
                />
                {errors.startTime && (
                  <Text id="schedule-time-error" size="1" className="task-scheduler__error">{errors.startTime}</Text>
                )}
              </Box>
            </Flex>
          </Flex>
        );

      case 'daily':
        return (
          <Flex direction="column" gap="3">
            <Text size="2" weight="medium" className="task-scheduler__section-title">
              {TasksStrings.SCHEDULER.RUN_DAILY}
            </Text>
            <Flex gap="3" wrap="wrap">
              <Box className="task-scheduler__field task-scheduler__field--date">
                <label htmlFor="schedule-date" className="task-scheduler__label">
                  <Calendar size={14} />
                  {TasksStrings.SCHEDULER.START_DATE}
                </label>
                <TextField.Root
                  id="schedule-date"
                  type="date"
                  value={dateValue}
                  onChange={(e) => handleDateChange(e.target.value)}
                  disabled={disabled}
                  aria-describedby={errors.startDate ? 'schedule-date-error' : undefined}
                  aria-invalid={errors.startDate ? true : undefined}
                />
                {errors.startDate && (
                  <Text id="schedule-date-error" size="1" className="task-scheduler__error">{errors.startDate}</Text>
                )}
              </Box>
              <Box className="task-scheduler__field task-scheduler__field--time">
                <label htmlFor="schedule-time" className="task-scheduler__label">
                  <Clock size={14} />
                  {TasksStrings.SCHEDULER.TIME}
                </label>
                <TextField.Root
                  id="schedule-time"
                  type="time"
                  value={value.startTime || '00:00'}
                  onChange={(e) => handleTimeChange(e.target.value)}
                  disabled={disabled}
                  autoComplete="off"
                  aria-describedby={errors.startTime ? 'schedule-time-error' : undefined}
                  aria-invalid={errors.startTime ? true : undefined}
                />
                <Text size="1" color="gray" mt="1">{TasksStrings.SCHEDULER.TIMES_SHOWN_LOCAL_TIMEZONE(timezone)}</Text>
                {errors.startTime && (
                  <Text id="schedule-time-error" size="1" className="task-scheduler__error">{errors.startTime}</Text>
                )}
              </Box>
            </Flex>
          </Flex>
        );

      case 'weekly':
        return (
          <Flex direction="column" gap="4">
            <Flex gap="3" wrap="wrap">
              <Box className="task-scheduler__field task-scheduler__field--date">
                <label htmlFor="schedule-date" className="task-scheduler__label">
                  <Calendar size={14} />
                  {TasksStrings.SCHEDULER.START_DATE}
                </label>
                <TextField.Root
                  id="schedule-date"
                  type="date"
                  value={dateValue}
                  onChange={(e) => handleDateChange(e.target.value)}
                  disabled={disabled}
                  aria-describedby={errors.startDate ? 'schedule-date-error' : undefined}
                  aria-invalid={errors.startDate ? true : undefined}
                />
                {errors.startDate && (
                  <Text id="schedule-date-error" size="1" className="task-scheduler__error">{errors.startDate}</Text>
                )}
              </Box>
              <Box className="task-scheduler__field task-scheduler__field--time">
                <label htmlFor="schedule-time" className="task-scheduler__label">
                  <Clock size={14} />
                  {TasksStrings.SCHEDULER.TIME}
                </label>
                <TextField.Root
                  id="schedule-time"
                  type="time"
                  value={value.startTime || '00:00'}
                  onChange={(e) => handleTimeChange(e.target.value)}
                  disabled={disabled}
                  autoComplete="off"
                  aria-describedby={errors.startTime ? 'schedule-time-error' : undefined}
                  aria-invalid={errors.startTime ? true : undefined}
                />
                <Text size="1" color="gray" mt="1">{TasksStrings.SCHEDULER.TIMES_SHOWN_LOCAL_TIMEZONE(timezone)}</Text>
                {errors.startTime && (
                  <Text id="schedule-time-error" size="1" className="task-scheduler__error">{errors.startTime}</Text>
                )}
              </Box>
            </Flex>
            <Box className="task-scheduler__days">
              <Text size="2" weight="medium" className="task-scheduler__section-title">
                {TasksStrings.SCHEDULER.DAYS_RUN}
              </Text>
              <Flex gap="3" wrap="wrap" className="task-scheduler__days-grid">
                {WEEKDAYS.map((day) => (
                  <label key={day.value} className="task-scheduler__day-checkbox">
                    <Checkbox
                      checked={(value.recurringDays || []).includes(day.value)}
                      onCheckedChange={() => handleDayToggle(day.value)}
                      disabled={disabled}
                    />
                    <Text size="2">{day.label}</Text>
                  </label>
                ))}
              </Flex>
              {errors.recurringDays && (
                <Text size="1" className="task-scheduler__error">{errors.recurringDays}</Text>
              )}
            </Box>
          </Flex>
        );

      case 'monthly':
        return (
          <Flex direction="column" gap="4">
            <Flex gap="3" wrap="wrap">
              <Box className="task-scheduler__field task-scheduler__field--date">
                <label htmlFor="schedule-date" className="task-scheduler__label">
                  <Calendar size={14} />
                  {TasksStrings.SCHEDULER.START_DATE}
                </label>
                <TextField.Root
                  id="schedule-date"
                  type="date"
                  value={dateValue}
                  onChange={(e) => handleDateChange(e.target.value)}
                  disabled={disabled}
                  aria-describedby={errors.startDate ? 'schedule-date-error' : undefined}
                  aria-invalid={errors.startDate ? true : undefined}
                />
                {errors.startDate && (
                  <Text id="schedule-date-error" size="1" className="task-scheduler__error">{errors.startDate}</Text>
                )}
              </Box>
              <Box className="task-scheduler__field task-scheduler__field--time">
                <label htmlFor="schedule-time" className="task-scheduler__label">
                  <Clock size={14} />
                  {TasksStrings.SCHEDULER.TIME}
                </label>
                <TextField.Root
                  id="schedule-time"
                  type="time"
                  value={value.startTime || '00:00'}
                  onChange={(e) => handleTimeChange(e.target.value)}
                  disabled={disabled}
                  autoComplete="off"
                  aria-describedby={errors.startTime ? 'schedule-time-error' : undefined}
                  aria-invalid={errors.startTime ? true : undefined}
                />
                <Text size="1" color="gray" mt="1">{TasksStrings.SCHEDULER.TIMES_SHOWN_LOCAL_TIMEZONE(timezone)}</Text>
                {errors.startTime && (
                  <Text id="schedule-time-error" size="1" className="task-scheduler__error">{errors.startTime}</Text>
                )}
              </Box>
            </Flex>
            <Box className="task-scheduler__days">
              <Text size="2" weight="medium" className="task-scheduler__section-title">
                {TasksStrings.SCHEDULER.DAYS_MONTH_RUN}
              </Text>
              <Grid columns="7" gap="2" className="task-scheduler__month-grid">
                {MONTH_DAYS.map((day) => (
                  <label 
                    key={day.value} 
                    className={`task-scheduler__month-day ${
                      (value.recurringDays || []).includes(day.value) 
                        ? 'task-scheduler__month-day--selected' 
                        : ''
                    }`}
                  >
                    <Checkbox
                      checked={(value.recurringDays || []).includes(day.value)}
                      onCheckedChange={() => handleDayToggle(day.value)}
                      disabled={disabled}
                      className="task-scheduler__month-checkbox"
                    />
                    <Text size="1">{day.label}</Text>
                  </label>
                ))}
              </Grid>
              {errors.recurringDays && (
                <Text size="1" className="task-scheduler__error">{errors.recurringDays}</Text>
              )}
            </Box>
          </Flex>
        );

      case 'advanced': {
        const cronResult = value.cronExpression
          ? validateCronExpression(value.cronExpression)
          : { valid: false };

        return (
          <Flex direction="column" gap="3">
            <SettingsTextInput
              name="cronExpression"
              label={TasksStrings.SCHEDULER.CRON_EXPRESSION}
              value={value.cronExpression || ''}
              onChange={handleCronChange}
              placeholder="0 0 * * * ?"
              helpText={TasksStrings.SCHEDULER.ENTER_QUARTZ_CRON_EXPRESSION_FORMAT}
              monospace
              error={errors.cronExpression || (
                value.cronExpression && !cronResult.valid ? cronResult.reason || TasksStrings.SCHEDULER.INVALID_CRON_EXPRESSION : ''
              )}
              required
              disabled={disabled}
            />
            {value.cronExpression && cronResult.valid && (
              <Text size="1" color="green" data-testid="cron-preview" className="task-scheduler__cron-preview">
                {describeCron(value.cronExpression) || TasksStrings.SCHEDULER.VALID_CRON_EXPRESSION}
              </Text>
            )}
            <Box className="task-scheduler__cron-help">
              <Text size="1" className="task-scheduler__cron-examples">
                <strong>{TasksStrings.SCHEDULER.EXAMPLES}</strong><br />
                <code>0 0 3 * * ?</code> - {TasksStrings.SCHEDULER.EXAMPLE_DAILY}<br />
                <code>0 0 */4 * * ?</code> - {TasksStrings.SCHEDULER.EXAMPLE_EVERY_4_HOURS}<br />
                <code>0 0 9 ? * MON-FRI</code> - {TasksStrings.SCHEDULER.EXAMPLE_WEEKDAYS}<br />
                <code>0 30 8 1 * ?</code> - {TasksStrings.SCHEDULER.EXAMPLE_MONTHLY}
              </Text>
            </Box>
          </Flex>
        );
      }

      default:
        return null;
    }
  };

  return (
    <Box className="task-scheduler">
      <SettingsSelect
        name="schedule"
        label={TasksStrings.SCHEDULER.SCHEDULE}
        value={value.schedule}
        onChange={handleScheduleChange}
        options={allowedSchedules
          ? SCHEDULE_OPTIONS.filter((o) => allowedSchedules.includes(o.value))
          : SCHEDULE_OPTIONS}
        placeholder={TasksStrings.SCHEDULER.SELECT_SCHEDULE_PLACEHOLDER}
        helpText={TasksStrings.SCHEDULER.CHOOSE_WHEN_TASK_SHOULD_RUN}
        error={errors.schedule}
        required
        disabled={disabled}
      />

      {value.schedule !== 'manual' && (
        <Box className="task-scheduler__fields">
          {renderScheduleFields()}
        </Box>
      )}

      {value.schedule === 'manual' && renderScheduleFields()}
    </Box>
  );
}

export default TaskScheduler;
