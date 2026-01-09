
package org.globalart.astrum.scheduling.schedule;

import java.util.Date;
import java.util.Set;

import org.globalart.astrum.scheduling.schedule.Monthly.CalendarDay;
import org.globalart.astrum.scheduling.schedule.Weekly.Weekday;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Support for {@link ScheduleFactory} implementations.
 *
 * @since 3.0
 */
public class ScheduleFactoryImpl
    implements ScheduleFactory
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Override
  public Manual manual() {
    return new Manual();
  }

  @Override
  public Now now() {
    return new Now();
  }

  @Override
  public Once once(final Date startAt) {
    checkNotNull(startAt);
    return new Once(startAt);
  }

  @Override
  public Hourly hourly(final Date startAt) {
    checkNotNull(startAt);
    return new Hourly(startAt);
  }

  @Override
  public Daily daily(final Date startAt) {
    checkNotNull(startAt);
    return new Daily(startAt);
  }

  @Override
  public Weekly weekly(final Date startAt, final Set<Weekday> daysToRun) {
    checkNotNull(startAt);
    checkNotNull(daysToRun);
    checkArgument(!daysToRun.isEmpty(), "No days of week set to run");
    return new Weekly(startAt, daysToRun);
  }

  @Override
  public Monthly monthly(final Date startAt, final Set<CalendarDay> daysToRun) {
    checkNotNull(startAt);
    checkNotNull(daysToRun);
    checkArgument(!daysToRun.isEmpty(), "No days of month set to run");
    return new Monthly(startAt, daysToRun);
  }

  @Override
  public Cron cron(final Date startAt, final String cronExpression) {
    checkNotNull(startAt);
    checkNotNull(cronExpression);
    return new Cron(startAt, cronExpression);
  }

  @Override
  public Cron cron(final Date startAt, final String cronExpression, final String zoneId) {
    checkNotNull(startAt);
    checkNotNull(cronExpression);
    checkNotNull(zoneId);
    return new Cron(startAt, cronExpression, zoneId);
  }
}
