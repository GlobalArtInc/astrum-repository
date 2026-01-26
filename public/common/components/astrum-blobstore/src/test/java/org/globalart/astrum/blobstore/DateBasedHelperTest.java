
package org.globalart.astrum.blobstore;

import java.time.Duration;
import java.time.OffsetDateTime;
import java.util.Map;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsInAnyOrder;

public class DateBasedHelperTest
{
  public static final OffsetDateTime NOW = OffsetDateTime.parse("2024-10-25T14:30:30Z");

  @Test
  public void testGeneratePrefixesMinutesLess30() {
    Duration duration = Duration.ofMinutes(1);
    Map<String, DateBasedHelper.DateInterval> prefixes = DateBasedHelper.generatePrefixes(NOW.minus(duration), NOW);
    assertThat(prefixes.keySet(), containsInAnyOrder("2024/10/25/14/29", "2024/10/25/14/30"));

    duration = Duration.ofMinutes(5);
    prefixes = DateBasedHelper.generatePrefixes(NOW.minus(duration), NOW);
    assertThat(prefixes.keySet(),
        containsInAnyOrder("2024/10/25/14/30", "2024/10/25/14/29", "2024/10/25/14/28", "2024/10/25/14/27",
            "2024/10/25/14/26", "2024/10/25/14/25"));
  }

  @Test
  public void testGeneratePrefixesMinutesOver30() {
    Duration duration = Duration.ofMinutes(31);
    Map<String, DateBasedHelper.DateInterval> prefixes = DateBasedHelper.generatePrefixes(NOW.minus(duration), NOW);
    assertThat(prefixes.keySet(), containsInAnyOrder("2024/10/25/13", "2024/10/25/14"));
  }

  @Test
  public void testGeneratePrefixesHoursLess24() {
    Duration duration = Duration.ofHours(3);
    Map<String, DateBasedHelper.DateInterval> prefixes = DateBasedHelper.generatePrefixes(NOW.minus(duration), NOW);
    assertThat(prefixes.keySet(),
        containsInAnyOrder("2024/10/25/14", "2024/10/25/13", "2024/10/25/11", "2024/10/25/12"));
  }

  @Test
  public void testGeneratePrefixesHoursOver24() {
    Duration duration = Duration.ofHours(25);
    Map<String, DateBasedHelper.DateInterval> prefixes = DateBasedHelper.generatePrefixes(NOW.minus(duration), NOW);
    assertThat(prefixes.keySet(), containsInAnyOrder("2024/10/25", "2024/10/24"));
  }

  @Test
  public void testGeneratePrefixesHoursOne() {
    Duration duration = Duration.ofHours(1);
    // sometimes current hour were not generated as prefix, so define specific value
    OffsetDateTime currentTime = OffsetDateTime.parse("2024-10-25T14:32:30Z");
    Map<String, DateBasedHelper.DateInterval> prefixes =
        DateBasedHelper.generatePrefixes(currentTime.minus(duration), currentTime);
    assertThat(prefixes.keySet(), containsInAnyOrder("2024/10/25/14", "2024/10/25/13"));
  }

  @Test
  public void testGeneratePrefixesMinutesOne() {
    Duration duration = Duration.ofMinutes(1);
    // sometimes current hour were not generated as prefix, so define specific value
    OffsetDateTime currentTime = OffsetDateTime.parse("2024-10-25T14:32:30Z");
    Map<String, DateBasedHelper.DateInterval> prefixes =
        DateBasedHelper.generatePrefixes(currentTime.minus(duration), currentTime);
    assertThat(prefixes.keySet(), containsInAnyOrder("2024/10/25/14/32", "2024/10/25/14/31"));
  }

  @Test
  public void testGeneratePrefixesDaysOne() {
    Duration duration = Duration.ofDays(1);
    // sometimes current hour were not generated as prefix, so define specific value
    OffsetDateTime currentTime = OffsetDateTime.parse("2024-10-25T14:32:30Z");
    Map<String, DateBasedHelper.DateInterval> prefixes =
        DateBasedHelper.generatePrefixes(currentTime.minus(duration), currentTime);
    assertThat(prefixes.keySet(), containsInAnyOrder("2024/10/25", "2024/10/24"));
  }

  @Test
  public void testGeneratePrefixesWithAdditionalOptions() {
    Duration duration = Duration.ofDays(1);
    String[] extraPrefixes = new String[]{"extra", "prefix"};

    // sometimes current hour were not generated as prefix, so define specific value
    OffsetDateTime currentTime = OffsetDateTime.parse("2024-10-25T14:32:30Z");
    Map<String, DateBasedHelper.DateInterval> prefixes =
        DateBasedHelper.generatePrefixes(currentTime.minus(duration), currentTime, extraPrefixes);
    assertThat(prefixes.keySet(), containsInAnyOrder("2024/10/25", "2024/10/24", "extra", "prefix"));
  }
}
