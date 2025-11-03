
package org.globalart.astrum.common.time;

import java.time.Duration;
import java.time.OffsetDateTime;

import org.joda.time.DateTime;
import org.junit.Test;

import static org.hamcrest.CoreMatchers.equalTo;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.globalart.astrum.common.time.DateHelper.toDateTime;
import static org.globalart.astrum.common.time.DateHelper.toJavaDuration;
import static org.globalart.astrum.common.time.DateHelper.toJodaDuration;
import static org.globalart.astrum.common.time.DateHelper.toOffsetDateTime;

public class DateHelperTest
{
  @Test
  public void toDateTimeTest() {
    OffsetDateTime offsetDateTime = OffsetDateTime.parse("2010-06-30T01:20+00:00");
    DateTime jodaDateTime = new DateTime("2010-06-30T01:20+00:00");
    assertThat(toDateTime(offsetDateTime).toInstant(), equalTo(jodaDateTime.toInstant()));
  }

  @Test
  public void toOffsetDateTimeTest() {
    OffsetDateTime offsetDateTime = OffsetDateTime.parse("2010-06-30T01:20+00:00");
    DateTime jodaDateTime = new DateTime("2010-06-30T01:20+00:00");
    assertThat(toOffsetDateTime(jodaDateTime).toInstant(), equalTo(offsetDateTime.toInstant()));
  }

  @Test
  public void toJavaDurationTest() {
    Duration javaDuration = Duration.ofHours(5);
    org.joda.time.Duration jodaDuration = org.joda.time.Duration.standardHours(5);
    assertThat(toJavaDuration(jodaDuration), equalTo(javaDuration));
  }

  @Test
  public void toJodaDurationTest() {
    Duration javaDuration = Duration.ofHours(5);
    org.joda.time.Duration jodaDuration = org.joda.time.Duration.standardHours(5);
    assertThat(toJodaDuration(javaDuration), equalTo(jodaDuration));
  }
}
