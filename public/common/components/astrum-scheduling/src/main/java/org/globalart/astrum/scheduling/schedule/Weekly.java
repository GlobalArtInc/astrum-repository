
package org.globalart.astrum.scheduling.schedule;

import java.util.Date;
import java.util.Set;
import java.util.function.Function;

/**
 * Schedule that repeats on same days of a week repeatedly.
 *
 * @see ScheduleFactory#weekly(Date, Set)
 */
public class Weekly
    extends Schedule
{
  public static final String TYPE = "weekly";

  /**
   * Representation of weekday.
   */
  public enum Weekday
  {
    SUN,
    MON,
    TUE,
    WED,
    THU,
    FRI,
    SAT;

    /**
     * Function to convert {@link Weekday} to strings.
     */
    public static final Function<Weekday, String> dayToString = Enum::name;

    /**
     * Function to convert string to {@link Weekday}.
     */
    public static final Function<String, Weekday> stringToDay = Weekday::valueOf;
  }

  public Weekly(final Date startAt, final Set<Weekday> daysToRun) {
    super(TYPE);
    set(SCHEDULE_START_AT, dateToString(startAt));
    set(SCHEDULE_DAYS_TO_RUN, setToCsv(daysToRun, Weekday.dayToString));
  }

  public Date getStartAt() {
    return stringToDate(get(SCHEDULE_START_AT));
  }

  public Set<Weekday> getDaysToRun() {
    return csvToSet(get(SCHEDULE_DAYS_TO_RUN), Weekday.stringToDay);
  }
}
