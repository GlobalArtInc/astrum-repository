
package org.globalart.astrum.repository.date;

import java.time.LocalDateTime;

import static org.globalart.astrum.scheduling.schedule.Weekly.Weekday.SAT;
import static org.globalart.astrum.scheduling.schedule.Weekly.Weekday.SUN;

/**
 * Utils for the astrum repository allowing perform specific timezone operations.
 *
 * @since 3.19
 */
public class TimeZoneUtils
{
  private TimeZoneUtils() {
    // empty
  }

  /**
   * The aim of this method to handle days of the week in case client and server are in different dates.
   * e.g. server date is (20/07/2019) and client date is (21/07/2019) and specific day of the week is specified.
   *
   * @param weekDayNumber it is ordinal number from enum Weekly.Weekday. First is SUN(0) e.g. SUN(0), MON(1), ...
   * @param clientDate Represent the client Date
   * @param serverDate Represent the client Date but in server TimeZone.
   * @return Returns transformed day of the week (one day back/forward/without changes) according client and server
   *         timezone.
   */
  public static int shiftWeekDay(
      final int weekDayNumber,
      final LocalDateTime clientDate,
      final LocalDateTime serverDate)
  {
    final int compare = clientDate.toLocalDate().compareTo(serverDate.toLocalDate());
    if (compare > 0) {
      return weekDayNumber - 1 < SUN.ordinal() ? SAT.ordinal() : weekDayNumber - 1;
    }
    else if (compare < 0) {
      return weekDayNumber + 1 > SAT.ordinal() ? SUN.ordinal() : weekDayNumber + 1;
    }
    return weekDayNumber;
  }

  /**
   * @param monthDay Represent ordinal number of the month day. 1, 2, 3 ... 31.
   * @param clientDate Represent the client Date
   * @param serverDate Represent the client Date but in server TimeZone.
   * @return Returns transformed day of the month (one day back/forward/without changes) according client and server
   *         timezone.
   */
  public static int shiftMonthDay(
      final int monthDay,
      final LocalDateTime clientDate,
      final LocalDateTime serverDate)
  {
    final int lengthOfMonth = serverDate.toLocalDate().lengthOfMonth();
    final int compare = clientDate.toLocalDate().compareTo(serverDate.toLocalDate());
    if (compare > 0) {
      return monthDay - 1 < 1 ? lengthOfMonth : monthDay - 1;
    }
    else if (compare < 0) {
      return monthDay + 1 > lengthOfMonth ? 1 : monthDay + 1;
    }
    return monthDay;
  }
}
