package org.globalart.astrum.scheduling.internal.resources;

import java.time.OffsetDateTime;
import java.util.Date;
import java.util.TimeZone;

import org.globalart.astrum.rest.ValidationErrorsException;
import org.globalart.astrum.scheduling.api.FrequencyXO;
import org.globalart.astrum.scheduling.schedule.Cron;
import org.globalart.astrum.scheduling.schedule.Hourly;
import org.globalart.astrum.scheduling.schedule.Manual;
import org.globalart.astrum.scheduling.schedule.Monthly;
import org.globalart.astrum.scheduling.schedule.Monthly.CalendarDay;
import org.globalart.astrum.scheduling.schedule.Once;
import org.globalart.astrum.scheduling.schedule.ScheduleFactoryImpl;
import org.globalart.astrum.scheduling.schedule.Weekly;
import org.globalart.astrum.scheduling.schedule.Weekly.Weekday;

import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.Matchers.instanceOf;
import static org.hamcrest.Matchers.is;
import static org.junit.jupiter.api.Assertions.assertThrows;

class TaskScheduleConverterTest
{
  /**
   * Checks cron expressions like the scheduler's factory does.
   */
  private static class CheckingScheduleFactory
      extends ScheduleFactoryImpl
  {
    @Override
    public Cron cron(final Date startAt, final String cronExpression) {
      if (!cronExpression.matches("[0-9*?/ ]+")) {
        throw new IllegalArgumentException("Invalid Cron expression: " + cronExpression);
      }
      return super.cron(startAt, cronExpression);
    }
  }

  private final TaskScheduleConverter underTest = new TaskScheduleConverter(new CheckingScheduleFactory());

  private TimeZone serverTimeZone;

  @BeforeEach
  void setServerTimeZone() {
    serverTimeZone = TimeZone.getDefault();
    TimeZone.setDefault(TimeZone.getTimeZone("UTC"));
  }

  @AfterEach
  void restoreServerTimeZone() {
    TimeZone.setDefault(serverTimeZone);
  }

  @Test
  void noFrequencyIsManual() {
    assertThat(underTest.asSchedule(null), instanceOf(Manual.class));
    assertThat(underTest.asSchedule(frequency("manual", null, null)), instanceOf(Manual.class));
  }

  @Test
  void startDateIsTruncatedToTheMinute() {
    long start = OffsetDateTime.parse("2030-01-02T03:04:05.678Z").toInstant().toEpochMilli();

    Hourly hourly = (Hourly) underTest.asSchedule(frequency("hourly", start, "+00:00"));

    assertThat(hourly.getStartAt().toInstant().toString(), is("2030-01-02T03:04:00Z"));
  }

  @Test
  void onceMustStartInTheFuture() {
    long future = System.currentTimeMillis() + 3_600_000L;
    assertThat(underTest.asSchedule(frequency("once", future, "+00:00")), instanceOf(Once.class));

    assertError(frequency("once", System.currentTimeMillis() - 60_000L, "+00:00"), "frequency.startDate");
    assertError(frequency("daily", null, "+00:00"), "frequency.startDate");
  }

  @Test
  void weeklyDaysStayWhenClientAndServerShareTheDate() {
    FrequencyXO frequency = frequency("weekly", epoch("2030-01-07T12:00:00Z"), "+00:00");
    frequency.setRecurringDays(new Integer[]{2, 4});

    Weekly weekly = (Weekly) underTest.asSchedule(frequency);

    assertThat(weekly.getDaysToRun(), containsInAnyOrder(Weekday.MON, Weekday.WED));
  }

  @Test
  void weeklyDaysMoveBackWhenTheClientIsADayAhead() {
    // 01:00 on Monday in the client's zone is still Sunday on the server
    FrequencyXO frequency = frequency("weekly", epoch("2030-01-07T01:00:00+03:00"), "+03:00");
    frequency.setRecurringDays(new Integer[]{1, 2});

    Weekly weekly = (Weekly) underTest.asSchedule(frequency);

    assertThat(weekly.getDaysToRun(), containsInAnyOrder(Weekday.SAT, Weekday.SUN));
  }

  @Test
  void monthlyDaysMoveWithTheDateAndKeepTheLastDay() {
    FrequencyXO ahead = frequency("monthly", epoch("2030-01-07T01:00:00+03:00"), "+03:00");
    ahead.setRecurringDays(new Integer[]{1, 15, 999});

    Monthly monthly = (Monthly) underTest.asSchedule(ahead);

    assertThat(monthly.getDaysToRun(), containsInAnyOrder(CalendarDay.day(31), CalendarDay.day(14),
        CalendarDay.lastDay()));

    FrequencyXO same = frequency("monthly", epoch("2030-02-07T12:00:00Z"), "+00:00");
    same.setRecurringDays(new Integer[]{31});
    assertThat(((Monthly) underTest.asSchedule(same)).getDaysToRun(), containsInAnyOrder(CalendarDay.day(31)));
  }

  @Test
  void invalidDaysAreRejected() {
    FrequencyXO weekly = frequency("weekly", epoch("2030-01-07T12:00:00Z"), "+00:00");
    assertError(weekly, "frequency.recurringDays");
    weekly.setRecurringDays(new Integer[]{8});
    assertError(weekly, "frequency.recurringDays");

    FrequencyXO monthly = frequency("monthly", epoch("2030-01-07T12:00:00Z"), "+00:00");
    monthly.setRecurringDays(new Integer[]{32});
    assertError(monthly, "frequency.recurringDays");
  }

  @Test
  void cronUsesTheClientTimeZone() {
    FrequencyXO frequency = frequency("cron", null, "+03:00");
    frequency.setCronExpression("0 0 1 * * ?");

    Cron cron = (Cron) underTest.asSchedule(frequency);

    assertThat(cron.getCronExpression(), is("0 0 1 * * ?"));
    assertThat(cron.getTimeZone().getRawOffset(), is(3 * 3_600_000));
  }

  @Test
  void invalidCronAndUnknownSchedulesAreRejected() {
    assertError(frequency("cron", null, "+00:00"), "frequency.cronExpression");

    FrequencyXO invalid = frequency("cron", null, "+00:00");
    invalid.setCronExpression("every day");
    assertError(invalid, "frequency.cronExpression");

    FrequencyXO badZone = frequency("cron", null, "Mars/Olympus");
    badZone.setCronExpression("0 0 1 * * ?");
    assertError(badZone, "frequency.timeZoneOffset");

    assertError(frequency("fortnightly", null, null), "frequency.schedule");
  }

  private void assertError(final FrequencyXO frequency, final String id) {
    ValidationErrorsException error =
        assertThrows(ValidationErrorsException.class, () -> underTest.asSchedule(frequency));
    assertThat(error.getValidationErrors().get(0).getId(), is(id));
  }

  private static FrequencyXO frequency(final String schedule, final Long startDate, final String timeZoneOffset) {
    FrequencyXO frequency = new FrequencyXO();
    frequency.setSchedule(schedule);
    frequency.setStartDate(startDate);
    frequency.setTimeZoneOffset(timeZoneOffset);
    return frequency;
  }

  private static long epoch(final String dateTime) {
    return OffsetDateTime.parse(dateTime).toInstant().toEpochMilli();
  }
}
