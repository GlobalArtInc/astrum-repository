
package org.globalart.astrum.scheduling.schedule;

import java.util.Date;

/**
 * Schedule that repeats daily at same time.
 *
 * @see ScheduleFactory#daily(Date)
 */
public class Daily
    extends Schedule
{
  public static final String TYPE = "daily";

  public Daily(final Date startAt) {
    super(TYPE);
    set(SCHEDULE_START_AT, dateToString(startAt));
  }

  public Date getStartAt() {
    return stringToDate(get(SCHEDULE_START_AT));
  }
}
