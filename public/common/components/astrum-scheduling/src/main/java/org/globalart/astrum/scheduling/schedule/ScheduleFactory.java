
package org.globalart.astrum.scheduling.schedule;

import java.util.Date;
import java.util.Set;

import org.globalart.astrum.scheduling.schedule.Monthly.CalendarDay;
import org.globalart.astrum.scheduling.schedule.Weekly.Weekday;

/**
 * Constructs {@link Schedule} instances.
 */
public interface ScheduleFactory
{
  Manual manual();

  Now now();

  Once once(Date startAt);

  Hourly hourly(Date startAt);

  Daily daily(Date startAt);

  Weekly weekly(Date startAt, Set<Weekday> daysToRun);

  Monthly monthly(Date startAt, Set<CalendarDay> daysToRun);

  Cron cron(Date startAt, String cronExpression);

  Cron cron(Date startAt, String cronExpression, String zoneId);
}
