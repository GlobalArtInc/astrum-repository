
package org.globalart.astrum.scheduling.schedule;

import java.util.Date;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Schedule that executes once at given time.
 *
 * @see ScheduleFactory#once(Date)
 */
public class Once
    extends Schedule
{
  public static final String TYPE = "once";

  public Once(final Date startAt) {
    super(TYPE);
    checkNotNull(startAt);
    set(SCHEDULE_START_AT, dateToString(startAt));
  }

  public Date getStartAt() {
    return stringToDate(get(SCHEDULE_START_AT));
  }
}
