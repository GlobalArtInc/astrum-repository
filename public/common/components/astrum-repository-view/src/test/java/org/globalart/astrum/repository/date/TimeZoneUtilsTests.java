
package org.globalart.astrum.repository.date;

import java.time.LocalDateTime;

import org.junit.Test;

import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.globalart.astrum.scheduling.schedule.Weekly.Weekday.*;

public class TimeZoneUtilsTests
{
  @Test
  public void dayOffTheWeekIsShiftedUp_ifServerIsDayAhead() {
    LocalDateTime clientDate = LocalDateTime.of(2019, 7, 21, 0, 0);
    LocalDateTime serverDate = LocalDateTime.of(2019, 7, 22, 0, 0);
    final int result = TimeZoneUtils.shiftWeekDay(TUE.ordinal(), clientDate, serverDate);
    assertThat(result, equalTo(WED.ordinal()));
  }

  @Test
  public void dayOffTheWeekIsShiftedDown_ifClientDayIsAhead() {
    LocalDateTime clientDate = LocalDateTime.of(2019, 7, 22, 0, 0);
    LocalDateTime serverDate = LocalDateTime.of(2019, 7, 21, 0, 0);
    final int result = TimeZoneUtils.shiftWeekDay(TUE.ordinal(), clientDate, serverDate);
    assertThat(result, equalTo(MON.ordinal()));
  }

  @Test
  public void dayOffTheWeekIsNotShifted_ifClientAndServerDayIsTheSame() {
    LocalDateTime clientDate = LocalDateTime.of(2019, 7, 22, 0, 0);
    LocalDateTime serverDate = LocalDateTime.of(2019, 7, 22, 0, 0);
    final int result = TimeZoneUtils.shiftWeekDay(TUE.ordinal(), clientDate, serverDate);
    assertThat(result, equalTo(TUE.ordinal()));
  }

  @Test
  public void dayOffTheWeekIsShiftedToSun_ifNowIsSatAndShiftUp() {
    LocalDateTime clientDate = LocalDateTime.of(2019, 7, 21, 0, 0);
    LocalDateTime serverDate = LocalDateTime.of(2019, 7, 22, 0, 0);
    final int result = TimeZoneUtils.shiftWeekDay(SAT.ordinal(), clientDate, serverDate);
    assertThat(result, equalTo(SUN.ordinal()));
  }

  @Test
  public void dayOffTheWeekIsShiftedToSat_ifNowIsSunAndShiftDown() {
    LocalDateTime clientDate = LocalDateTime.of(2019, 7, 22, 0, 0);
    LocalDateTime serverDate = LocalDateTime.of(2019, 7, 21, 0, 0);
    final int result = TimeZoneUtils.shiftWeekDay(SUN.ordinal(), clientDate, serverDate);
    assertThat(result, equalTo(SAT.ordinal()));
  }

  @Test
  public void dayOffTheWeekIsShiftedToWed_ifServerIsInJulyAndClientIsInAugust() {
    LocalDateTime clientDate = LocalDateTime.of(2019, 8, 1, 0, 0);
    LocalDateTime serverDate = LocalDateTime.of(2019, 7, 31, 0, 0);
    final int result = TimeZoneUtils.shiftWeekDay(THU.ordinal(), clientDate, serverDate);
    assertThat(result, equalTo(WED.ordinal()));
  }

  @Test
  public void dayOffTheMonthIsSheftedUp_ifServerFutureAndClientPast() {
    LocalDateTime clientDate = LocalDateTime.of(2019, 7, 20, 0, 0);
    LocalDateTime serverDate = LocalDateTime.of(2019, 7, 21, 0, 0);
    final int result = TimeZoneUtils.shiftMonthDay(20, clientDate, serverDate);
    assertThat(result, equalTo(21));
  }

  @Test
  public void dayOffTheMonthIsSheftedUp_ifServerPastAndClientFuture() {
    LocalDateTime clientDate = LocalDateTime.of(2019, 7, 21, 0, 0);
    LocalDateTime serverDate = LocalDateTime.of(2019, 7, 20, 0, 0);
    final int result = TimeZoneUtils.shiftMonthDay(21, clientDate, serverDate);
    assertThat(result, equalTo(20));
  }

  @Test
  public void dayOffTheMonthIsSheftedUp_ifServerFutureMonthAndClientPastMonth() {
    LocalDateTime clientDate = LocalDateTime.of(2019, 7, 31, 0, 0);
    LocalDateTime serverDate = LocalDateTime.of(2019, 8, 1, 0, 0);
    final int result = TimeZoneUtils.shiftMonthDay(31, clientDate, serverDate);
    assertThat(result, equalTo(1));
  }

  @Test
  public void dayOffTheMonthIsSheftedUp_ifServerPastMonthAndClientFutureMonth() {
    LocalDateTime clientDate = LocalDateTime.of(2019, 8, 1, 0, 0);
    LocalDateTime serverDate = LocalDateTime.of(2019, 7, 31, 0, 0);
    final int result = TimeZoneUtils.shiftMonthDay(1, clientDate, serverDate);
    assertThat(result, equalTo(31));
  }
}
