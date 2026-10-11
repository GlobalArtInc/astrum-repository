package org.globalart.astrum.scheduling.internal.resources;

import java.time.DateTimeException;
import java.time.Instant;
import java.time.LocalDate;
import java.time.ZoneId;
import java.time.ZoneOffset;
import java.util.Arrays;
import java.util.Calendar;
import java.util.Date;
import java.util.Objects;
import java.util.Set;

import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.rest.ValidationErrorsException;
import org.globalart.astrum.scheduling.api.FrequencyXO;
import org.globalart.astrum.scheduling.schedule.Monthly.CalendarDay;
import org.globalart.astrum.scheduling.schedule.Schedule;
import org.globalart.astrum.scheduling.schedule.ScheduleFactory;
import org.globalart.astrum.scheduling.schedule.Weekly.Weekday;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.stream.Collectors.toSet;

/**
 * Turns the frequency of a task sent to the REST API into a {@link Schedule}, as the Classic task form does: the start
 * date and the days are given in the client's time zone.
 */
class TaskScheduleConverter
{
  static final String FREQUENCY = "frequency.";

  private static final int LAST_DAY_OF_MONTH = 999;

  private final ScheduleFactory scheduleFactory;

  TaskScheduleConverter(final ScheduleFactory scheduleFactory) {
    this.scheduleFactory = checkNotNull(scheduleFactory);
  }

  Schedule asSchedule(final FrequencyXO frequency) {
    String type = frequency == null || Strings2.isBlank(frequency.getSchedule()) ? "manual" : frequency.getSchedule();
    switch (type) {
      case "manual":
        return scheduleFactory.manual();
      case "cron":
      case "advanced":
        return cron(frequency);
      case "once":
        Date startAt = startDate(frequency);
        if (!startAt.after(new Date())) {
          throw new ValidationErrorsException(FREQUENCY + "startDate", "Start date must be in the future");
        }
        return scheduleFactory.once(startAt);
      case "hourly":
        return scheduleFactory.hourly(startDate(frequency));
      case "daily":
        return scheduleFactory.daily(startDate(frequency));
      case "weekly":
        return scheduleFactory.weekly(startDate(frequency), weekdays(frequency));
      case "monthly":
        return scheduleFactory.monthly(startDate(frequency), calendarDays(frequency));
      default:
        throw new ValidationErrorsException(FREQUENCY + "schedule", "Unknown schedule: " + type);
    }
  }

  private Schedule cron(final FrequencyXO frequency) {
    if (Strings2.isBlank(frequency.getCronExpression())) {
      throw new ValidationErrorsException(FREQUENCY + "cronExpression", "Cron expression is required");
    }
    String expression = frequency.getCronExpression().trim();
    try {
      // only the factory method without a time zone checks the expression
      scheduleFactory.cron(new Date(), expression);
      return scheduleFactory.cron(new Date(), expression, clientZone(frequency).getId());
    }
    catch (IllegalArgumentException e) {
      throw new ValidationErrorsException(FREQUENCY + "cronExpression",
          e.getMessage() != null ? e.getMessage() : "Invalid cron expression");
    }
  }

  private static Date startDate(final FrequencyXO frequency) {
    if (frequency.getStartDate() == null) {
      throw new ValidationErrorsException(FREQUENCY + "startDate", "Start date is required");
    }
    Calendar date = Calendar.getInstance();
    date.setTimeInMillis(frequency.getStartDate());
    date.set(Calendar.SECOND, 0);
    date.set(Calendar.MILLISECOND, 0);
    return date.getTime();
  }

  private static ZoneOffset clientZone(final FrequencyXO frequency) {
    if (Strings2.isBlank(frequency.getTimeZoneOffset())) {
      return ZoneOffset.UTC;
    }
    try {
      return ZoneOffset.of(frequency.getTimeZoneOffset().trim());
    }
    catch (DateTimeException e) {
      throw new ValidationErrorsException(FREQUENCY + "timeZoneOffset",
          "Invalid time zone offset: " + frequency.getTimeZoneOffset());
    }
  }

  /**
   * How many days the start date in the server's time zone lies after the start date in the client's: a day picked by
   * the client moves by as much.
   */
  private static int dayShift(final FrequencyXO frequency) {
    Instant start = Instant.ofEpochMilli(frequency.getStartDate());
    LocalDate client = LocalDate.ofInstant(start, clientZone(frequency));
    LocalDate server = LocalDate.ofInstant(start, ZoneId.systemDefault());
    return Integer.signum(server.compareTo(client));
  }

  private static Integer[] recurringDays(final FrequencyXO frequency) {
    Integer[] days = frequency.getRecurringDays();
    if (days == null || days.length == 0 || Arrays.stream(days).anyMatch(Objects::isNull)) {
      throw new ValidationErrorsException(FREQUENCY + "recurringDays", "At least one day is required");
    }
    return days;
  }

  private Set<Weekday> weekdays(final FrequencyXO frequency) {
    Integer[] days = recurringDays(frequency);
    int shift = dayShift(frequency);
    Weekday[] weekdays = Weekday.values();
    for (Integer day : days) {
      if (day < 1 || day > weekdays.length) {
        throw new ValidationErrorsException(FREQUENCY + "recurringDays", "Invalid day of the week: " + day);
      }
    }
    return Arrays.stream(days)
        .map(day -> weekdays[Math.floorMod(day - 1 + shift, weekdays.length)])
        .collect(toSet());
  }

  private Set<CalendarDay> calendarDays(final FrequencyXO frequency) {
    Integer[] days = recurringDays(frequency);
    int shift = dayShift(frequency);
    int lengthOfMonth = LocalDate.ofInstant(Instant.ofEpochMilli(frequency.getStartDate()), ZoneId.systemDefault())
        .lengthOfMonth();
    for (Integer day : days) {
      if (day != LAST_DAY_OF_MONTH && (day < 1 || day > 31)) {
        throw new ValidationErrorsException(FREQUENCY + "recurringDays", "Invalid day of the month: " + day);
      }
    }
    return Arrays.stream(days)
        .map(day -> day == LAST_DAY_OF_MONTH
            ? CalendarDay.lastDay()
            : CalendarDay.day(shiftDay(day, shift, lengthOfMonth)))
        .collect(toSet());
  }

  private static int shiftDay(final int day, final int shift, final int lengthOfMonth) {
    if (shift < 0) {
      return day - 1 < 1 ? lengthOfMonth : day - 1;
    }
    if (shift > 0) {
      return day + 1 > lengthOfMonth ? 1 : day + 1;
    }
    return day;
  }
}
