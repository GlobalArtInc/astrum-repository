
package org.globalart.astrum.scheduling.schedule;

import java.util.Date;

/**
 * Schedule that repeats every hour at same minute of hour.
 *
 * @see ScheduleFactory#hourly(Date)
 */
public class Hourly
    extends Schedule
{
  public static final String TYPE = "hourly";

  public Hourly(final Date startAt) {
    super(TYPE);
    set(SCHEDULE_START_AT, dateToString(startAt));
  }

  public Date getStartAt() {
    return stringToDate(get(SCHEDULE_START_AT));
  }
}
