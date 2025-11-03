
package org.globalart.astrum.common.time;

import java.util.concurrent.TimeUnit;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.junit.Assert.assertEquals;

/**
 * Tests for {@link Time}.
 */
public class TimeTest
{
  @Test(expected = NullPointerException.class)
  public void nullUnit() {
    new Time(1, null);
  }

  @Test
  public void testTimeConversions() {
    Time oneDay = Time.days(1);

    assertEquals(1 * 24 * 60 * 60 * 1000 * 1000L, oneDay.toMicros());
    assertEquals(1 * 24 * 60 * 60 * 1000L, oneDay.toMillis());
    assertEquals(1 * 24 * 60 * 60L, oneDay.toSeconds());
    assertEquals(1 * 24 * 60L, oneDay.toMinutes());
    assertEquals(1 * 24L, oneDay.toHours());
    assertEquals(1L, oneDay.toDays());

    assertEquals(Time.micros(1 * 24 * 60 * 60 * 1000 * 1000L).toDays(), oneDay.value());
    assertEquals(Time.millis(1 * 24 * 60 * 60 * 1000L).toDays(), oneDay.value());
    assertEquals(Time.seconds(1 * 24 * 60 * 60L).toDays(), oneDay.value());
    assertEquals(Time.minutes(1 * 24 * 60L).toDays(), oneDay.value());
    assertEquals(Time.hours(1 * 24L).toDays(), oneDay.value());
    assertEquals(Time.days(1L).toDays(), oneDay.value());
  }

  @Test
  public void parse_nS() throws Exception {
    Time time = Time.parse("1s");
    assertThat(time, equalTo(Time.seconds(1)));
  }

  @Test
  public void parse_n_S() throws Exception {
    Time time = Time.parse("1 s");
    assertThat(time, equalTo(Time.seconds(1)));
  }

  @Test
  public void parse_n_S_() throws Exception {
    Time time = Time.parse("1 s ");
    assertThat(time, equalTo(Time.seconds(1)));
  }

  @Test
  public void parse__n_S_() throws Exception {
    Time time = Time.parse(" 1 s ");
    assertThat(time, equalTo(Time.seconds(1)));
  }

  @Test
  public void parse_nSec() throws Exception {
    Time time = Time.parse("1sec");
    assertThat(time, equalTo(Time.seconds(1)));
  }

  @Test
  public void parse_nM() throws Exception {
    Time time = Time.parse("1m");
    assertThat(time, equalTo(Time.minutes(1)));
  }

  @Test
  public void parse_nMin() throws Exception {
    Time time = Time.parse("1min");
    assertThat(time, equalTo(Time.minutes(1)));
  }

  @Test
  public void asSeconds() {
    Time time = Time.minutes(2).asSeconds();
    assertThat(time.unit(), equalTo(TimeUnit.SECONDS));
    assertThat(time.value(), equalTo((2 * 60L)));
  }

  @Test
  public void toMillisI() throws Exception {
    int n = Time.seconds(1).toMillisI();
    assertThat(n, equalTo(1000));
  }
}
