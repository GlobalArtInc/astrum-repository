
package org.globalart.astrum.common.time;

import java.time.Duration;
import java.time.Instant;
import java.time.LocalDateTime;
import java.time.OffsetDateTime;
import java.time.ZoneId;
import java.time.ZoneOffset;
import java.time.format.DateTimeFormatter;
import java.util.Date;
import java.util.List;
import java.util.Optional;
import java.util.stream.Collectors;
import javax.annotation.Nullable;

import org.joda.time.DateTime;
import org.joda.time.DateTimeZone;

import static java.time.temporal.ChronoUnit.DAYS;
import static java.time.temporal.ChronoUnit.HOURS;
import static java.time.temporal.ChronoUnit.MINUTES;
import static java.time.temporal.ChronoUnit.MONTHS;
import static java.time.temporal.ChronoUnit.SECONDS;
import static java.time.temporal.ChronoUnit.WEEKS;

/**
 * Helper for {@link DateTime} and {@link Date} types.
 *
 * @since 3.0
 */
public class DateHelper
{
  private static final long MILLIS_PER_MONTH = MONTHS.getDuration().toMillis();

  private static final long MILLIS_PER_WEEK = WEEKS.getDuration().toMillis();

  private static final long MILLIS_PER_DAY = DAYS.getDuration().toMillis();

  private static final long MILLIS_PER_HOUR = HOURS.getDuration().toMillis();

  private static final long MILLIS_PER_MINUTE = MINUTES.getDuration().toMillis();

  private static final long MILLIS_PER_SECOND = SECONDS.getDuration().toMillis();

  private DateHelper() {
    // empty
  }

  /**
   * Copy given date.
   */
  @Nullable
  public static Date copy(@Nullable final Date date) {
    if (date != null) {
      return new Date(date.getTime());
    }
    return null;
  }

  /**
   * Converts input {@link Date} to {@link DateTime}, or {@code null} if input is {@code null}.
   */
  @Nullable
  public static DateTime toDateTime(@Nullable final Date date) {
    if (date == null) {
      return null;
    }
    return new DateTime(date.getTime());
  }

  /**
   * Converts input {@link DateTime} to {@link Date}, or {@code null} if input is {@code null}.
   */
  @Nullable
  public static Date toDate(@Nullable final DateTime dateTime) {
    if (dateTime == null) {
      return null;
    }
    return dateTime.toDate();
  }

  /**
   * Converts input {@link Long} to {@link Date}, or {@code null} if input is {@code null}.
   */
  @Nullable
  public static Date toDate(@Nullable final Long timestamp) {
    if (timestamp == null || timestamp <= 0) {
      return null;
    }
    Instant instant = Instant.ofEpochMilli(timestamp);
    if (instant.equals(Instant.EPOCH)) {
      return null;
    }
    return new Date(timestamp);
  }

  /**
   * Converts input {@link OffsetDateTime} to {@link Date}, or {@code null} if input is {@code null}.
   */
  @Nullable
  public static Date toDate(@Nullable final OffsetDateTime dateTime) {
    if (dateTime == null) {
      return null;
    }
    return new Date(dateTime.toInstant().toEpochMilli());
  }

  /**
   * @since 3.25
   */
  public static DateTime toDateTime(final OffsetDateTime offsetDateTime) {
    return new DateTime(offsetDateTime.toInstant().toEpochMilli(), DateTimeZone.UTC);
  }

  /**
   * @since 3.25
   */
  public static OffsetDateTime toOffsetDateTime(final DateTime dateTime) {
    return Instant.ofEpochMilli(dateTime.getMillis()).atOffset(ZoneOffset.UTC);
  }

  public static OffsetDateTime toOffsetDateTime(final Date date) {
    return Instant.ofEpochMilli(date.getTime()).atOffset(ZoneOffset.UTC);
  }

  /**
   * @since 3.24
   */
  public static Duration toJavaDuration(final org.joda.time.Duration jodaDuration) {
    return Duration.ofMillis(jodaDuration.getMillis());
  }

  /**
   * @since 3.24
   */
  public static org.joda.time.Duration toJodaDuration(final Duration javaDuration) {
    return org.joda.time.Duration.millis(javaDuration.toMillis());
  }

  /**
   * Converts an Optional<OffsetDateTime> to a Date.
   */
  public static Date optionalOffsetToDate(final Optional<OffsetDateTime> offsetDateTime) {
    return offsetDateTime.map(OffsetDateTime::toInstant)
        .map(Date::from)
        .orElse(null);
  }

  /**
   * Converts an OffsetDateTime to a Date.
   */
  public static Date offsetToDate(final OffsetDateTime offsetDateTime) {
    return Date.from(offsetDateTime.toInstant());
  }

  /**
   * Receives a list of dates and returns the oldest one. If the list is null or empty, returns null.
   */
  public static Date oldestDateFromLongs(final List<Long> dates) {
    if (dates == null || dates.isEmpty()) {
      return null;
    }
    List<Date> dateList = dates.stream().map(DateHelper::toDate).collect(Collectors.toList());
    return oldestDate(dateList);
  }

  /**
   * Receives a list of dates and returns the oldest one. If the list is null or empty, returns null.
   */
  public static Date oldestDate(final List<Date> dates) {
    if (dates == null || dates.isEmpty()) {
      return null;
    }
    return dates.stream().filter(date -> date != null).min(Date::compareTo).orElse(null);
  }

  /**
   * Receives two dates and returns the number of days elapsed between them.
   */
  public static int daysElapsed(final Date date1, final Date date2) {
    if (date1 == null || date2 == null) {
      return 0;
    }
    return (int) ((date2.getTime() - date1.getTime()) / MILLIS_PER_DAY);
  }

  /**
   * Formats an Instant to a String in ISO-8601 format.
   */
  public static String isoFormatter(Instant instant) {
    if (instant == null) {
      return null;
    }
    LocalDateTime localDateTime = instant.atZone(ZoneId.systemDefault()).toLocalDateTime();
    return localDateTime.format(DateTimeFormatter.ISO_LOCAL_DATE_TIME);
  }

  public static String toDurationString(final long duration) {
    StringBuilder stringBuilder = new StringBuilder();
    long remainder = toDurationString(stringBuilder, "month", MILLIS_PER_MONTH, duration);
    remainder = toDurationString(stringBuilder, "week", MILLIS_PER_WEEK, remainder);
    remainder = toDurationString(stringBuilder, "day", MILLIS_PER_DAY, remainder);
    remainder = toDurationString(stringBuilder, "hour", MILLIS_PER_HOUR, remainder);
    remainder = toDurationString(stringBuilder, "minute", MILLIS_PER_MINUTE, remainder);
    toDurationString(stringBuilder, "second", MILLIS_PER_SECOND, remainder);

    return stringBuilder.toString();
  }

  private static long toDurationString(
      final StringBuilder stringBuilder,
      final String period,
      final long periodInMillis,
      final long duration)
  {
    long count = duration / periodInMillis;
    long remainder = duration % periodInMillis;

    if (count > 0) {
      if (!stringBuilder.isEmpty()) {
        stringBuilder.append(" ");
      }
      stringBuilder.append(count).append(" ").append(period).append(count > 1 ? "s" : "");
    }
    return remainder;
  }
}
