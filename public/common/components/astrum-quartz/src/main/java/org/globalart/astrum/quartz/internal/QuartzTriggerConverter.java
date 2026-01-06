
package org.globalart.astrum.quartz.internal;

import java.util.Calendar;
import java.util.Date;
import java.util.Map;
import java.util.Set;
import java.util.stream.Collectors;

import org.globalart.astrum.scheduling.schedule.Cron;
import org.globalart.astrum.scheduling.schedule.Daily;
import org.globalart.astrum.scheduling.schedule.Hourly;
import org.globalart.astrum.scheduling.schedule.Manual;
import org.globalart.astrum.scheduling.schedule.Monthly;
import org.globalart.astrum.scheduling.schedule.Monthly.CalendarDay;
import org.globalart.astrum.scheduling.schedule.Now;
import org.globalart.astrum.scheduling.schedule.Once;
import org.globalart.astrum.scheduling.schedule.Schedule;
import org.globalart.astrum.scheduling.schedule.ScheduleFactory;
import org.globalart.astrum.scheduling.schedule.Weekly;
import org.globalart.astrum.scheduling.schedule.Weekly.Weekday;

import com.google.common.annotations.VisibleForTesting;
import org.quartz.CronScheduleBuilder;
import org.quartz.JobDataMap;
import org.quartz.SimpleScheduleBuilder;
import org.quartz.Trigger;
import org.quartz.TriggerBuilder;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.quartz.TriggerBuilder.newTrigger;
import static org.globalart.astrum.scheduling.schedule.Schedule.SCHEDULE_DAYS_TO_RUN;
import static org.globalart.astrum.scheduling.schedule.Schedule.SCHEDULE_START_AT;
import static org.globalart.astrum.scheduling.schedule.Schedule.csvToSet;
import static org.globalart.astrum.scheduling.schedule.Schedule.stringToDate;

/**
 * Provides conversion between Quartz {@link Trigger} and Astrum {@link Schedule}.
 *
 * @since 3.0
 */
public class QuartzTriggerConverter
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final ScheduleFactory scheduleFactory;

  public QuartzTriggerConverter(final ScheduleFactory scheduleFactory) {
    this.scheduleFactory = checkNotNull(scheduleFactory);
  }

  /**
   * Converts a Astrum {@link Schedule} into a Quartz {@link Trigger}.
   */
  public TriggerBuilder convert(final Schedule schedule) {
    checkNotNull(schedule);

    TriggerBuilder triggerBuilder;

    if (schedule instanceof Cron) {
      Cron s = (Cron) schedule;

      triggerBuilder = newTrigger()
          .startAt(s.getStartAt())
          .withSchedule(CronScheduleBuilder.cronSchedule(s.getCronExpression()).inTimeZone(s.getTimeZone()));
    }
    else if (schedule instanceof Now) {
      triggerBuilder = newTrigger()
          .startNow();
    }
    else if (schedule instanceof Once) {
      Once s = (Once) schedule;

      triggerBuilder = newTrigger()
          .startAt(s.getStartAt());
    }
    else if (schedule instanceof Hourly) {
      Hourly s = (Hourly) schedule;

      triggerBuilder = newTrigger()
          .startAt(s.getStartAt())
          .withSchedule(SimpleScheduleBuilder.repeatHourlyForever(1));
    }
    else if (schedule instanceof Daily) {
      Daily s = (Daily) schedule;

      triggerBuilder = newTrigger()
          .startAt(s.getStartAt())
          .withSchedule(SimpleScheduleBuilder.repeatHourlyForever(24));
    }
    else if (schedule instanceof Weekly) {
      Weekly s = (Weekly) schedule;

      String daysToRun = s.getDaysToRun()
          .stream()
          .map(Weekday.dayToString)
          .collect(Collectors.joining(","));

      triggerBuilder = newTrigger()
          .startAt(s.getStartAt())
          .withSchedule(cron(s.getStartAt(), "? * " + daysToRun));
    }
    else if (schedule instanceof Monthly) {
      Monthly s = (Monthly) schedule;

      Set<CalendarDay> daysToRun = s.getDaysToRun();
      boolean lastDayOfMonth = daysToRun.remove(CalendarDay.lastDay());

      // quartz does not support use of "L" along with days!
      if (!lastDayOfMonth) {
        String daysToRunStr = daysToRun.stream()
            .map(CalendarDay.dayToString)
            .collect(Collectors.joining(","));

        triggerBuilder = newTrigger()
            .startAt(s.getStartAt())
            .withSchedule(cron(s.getStartAt(), daysToRunStr + " * ?"));
      }
      else {
        triggerBuilder = newTrigger()
            .startAt(s.getStartAt())
            .withSchedule(cron(s.getStartAt(), "L * ?"));
      }
    }
    else if (schedule instanceof Manual) {
      // this looks awkward, but is needed to maintain job:trigger 1:1 ratio
      // that would introduce lot of exceptional branches handling trigger==null in code
      triggerBuilder = newTrigger()
          .startAt(new Date(Long.MAX_VALUE));
    }
    else {
      throw new IllegalArgumentException("Schedule unknown: " + schedule.getType());
    }

    // store all the schedule properties for opposite conversion
    for (Map.Entry<String, String> entry : schedule.asMap().entrySet()) {
      triggerBuilder.usingJobData(entry.getKey(), entry.getValue());
    }

    return triggerBuilder;
  }

  /**
   * Helper to build a cron-schedule.
   */
  private CronScheduleBuilder cron(final Date date, final String patternSuffix) {
    return CronScheduleBuilder.cronSchedule(cronTimeParts(date) + " " + patternSuffix);
  }

  /**
   * Returns CRON pattern prefix for date including minute and hour, at second {@code 0}.
   */
  @VisibleForTesting
  static String cronTimeParts(final Date date) {
    Calendar calendar = Calendar.getInstance();
    calendar.setTime(date);
    return String.format("0 %d %d", calendar.get(Calendar.MINUTE), calendar.get(Calendar.HOUR_OF_DAY));
  }

  /**
   * Converts Quartz {@link Trigger} into Astrum {@link Schedule}.
   *
   * Only converts triggers created initially from schedulers,
   * as the {@link Trigger#getJobDataMap()} is used as the source of configuration.
   */
  public Schedule convert(final Trigger trigger) {
    checkNotNull(trigger);

    JobDataMap jobData = trigger.getJobDataMap();
    String type = jobData.getString(Schedule.SCHEDULE_TYPE);

    if (Cron.TYPE.equals(type)) {
      Date startAt = stringToDate(jobData.getString(SCHEDULE_START_AT));
      String cronExpression = jobData.getString(Cron.SCHEDULE_CRON_EXPRESSION);
      return scheduleFactory.cron(startAt, cronExpression);
    }
    else if (Now.TYPE.equals(type)) {
      return scheduleFactory.now();
    }
    else if (Once.TYPE.equals(type)) {
      Date startAt = stringToDate(jobData.getString(SCHEDULE_START_AT));
      return scheduleFactory.once(startAt);
    }
    else if (Hourly.TYPE.equals(type)) {
      Date startAt = stringToDate(jobData.getString(SCHEDULE_START_AT));
      return scheduleFactory.hourly(startAt);
    }
    else if (Daily.TYPE.equals(type)) {
      Date startAt = stringToDate(jobData.getString(SCHEDULE_START_AT));
      return scheduleFactory.daily(startAt);
    }
    else if (Weekly.TYPE.equals(type)) {
      Date startAt = stringToDate(jobData.getString(SCHEDULE_START_AT));
      Set<Weekday> daysToRun = csvToSet(jobData.getString(SCHEDULE_DAYS_TO_RUN), Weekday.stringToDay);
      return scheduleFactory.weekly(startAt, daysToRun);
    }
    else if (Monthly.TYPE.equals(type)) {
      Date startAt = stringToDate(jobData.getString(SCHEDULE_START_AT));
      Set<CalendarDay> daysToRun = csvToSet(jobData.getString(SCHEDULE_DAYS_TO_RUN), CalendarDay.stringToDay);
      return scheduleFactory.monthly(startAt, daysToRun);
    }
    else if (Manual.TYPE.equals(type)) {
      return scheduleFactory.manual();
    }
    else {
      throw new IllegalArgumentException("Trigger unknown key: '" + trigger.getKey() + "', type: '" + type + "'");
    }
  }
}
