
package org.globalart.astrum.scheduling.schedule;

import java.util.Date;
import java.util.TimeZone;

/**
 * Schedule that accepts cron expression.
 *
 * @see ScheduleFactory#cron(Date, String)
 */
public class Cron
    extends Schedule
{
  public static final String TYPE = "cron";

  public static final String SCHEDULE_CRON_EXPRESSION = "schedule.cronExpression";

  public Cron(final Date startAt, final String cronExpression) {
    this(startAt, cronExpression, TimeZone.getDefault().getID());
  }

  public Cron(final Date startAt, final String cronExpression, final String zoneId) {
    super(TYPE);
    set(SCHEDULE_START_AT, dateToString(startAt));
    set(SCHEDULE_CRON_EXPRESSION, cronExpression);
    set(SCHEDULE_CLIENT_TIME_ZONE, zoneId);
  }

  public Date getStartAt() {
    return stringToDate(get(SCHEDULE_START_AT));
  }

  public String getCronExpression() {
    return get(SCHEDULE_CRON_EXPRESSION);
  }

  public TimeZone getTimeZone() {
    return stringToTimeZone(get(SCHEDULE_CLIENT_TIME_ZONE));
  }

}
