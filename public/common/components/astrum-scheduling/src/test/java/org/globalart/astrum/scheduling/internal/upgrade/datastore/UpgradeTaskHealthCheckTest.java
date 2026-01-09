
package org.globalart.astrum.scheduling.internal.upgrade.datastore;

import java.time.Duration;
import java.time.OffsetDateTime;
import java.util.List;
import java.util.Map;

import com.codahale.metrics.health.HealthCheck.Result;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.mockito.ArgumentCaptor;
import org.mockito.Captor;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.greaterThanOrEqualTo;
import static org.hamcrest.Matchers.lessThanOrEqualTo;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class UpgradeTaskHealthCheckTest

{
  private static final Duration ONE_DAY = Duration.ofDays(1);

  @Captor
  ArgumentCaptor<OffsetDateTime> captor;

  @Mock
  UpgradeTaskStore store;

  UpgradeTaskHealthCheck underTest;

  @BeforeEach
  void setup() {
    underTest = new UpgradeTaskHealthCheck(store, ONE_DAY, Duration.ofMinutes(5));
  }

  @Test
  void testCheck() {
    OffsetDateTime lowerBound = OffsetDateTime.now().minus(ONE_DAY);
    when(store.browse(any())).thenReturn(List.of());
    Result result = underTest.execute();
    OffsetDateTime upperBound = OffsetDateTime.now().minus(ONE_DAY);

    assertTrue(result.isHealthy());
    // running again should not call the store again until the cache expires
    underTest.execute();
    verify(store).browse(captor.capture());

    assertBetween(captor.getValue(), lowerBound, upperBound);
  }

  @Test
  void testCheck_fail() {
    when(store.browse(any())).thenReturn(List.of(new UpgradeTaskData("gibberish", Map.of(".typeId", "task-one"))));
    Result result = underTest.execute();

    assertFalse(result.isHealthy());
    assertThat(result.getMessage(), containsString("task-one"));

    // running again should not call the store again until the cache expires
    underTest.execute();
    verify(store).browse(captor.capture());
  }

  @Test
  void testCheck_cacheExpiry() throws InterruptedException {
    when(store.browse(any())).thenReturn(List.of());
    underTest = new UpgradeTaskHealthCheck(store, ONE_DAY, Duration.ofMillis(1));

    underTest.execute();
    // Sleep longer than the cache duration
    Thread.sleep(2);
    // running again after cache expiration should trigger a reload
    underTest.execute();

    verify(store, times(2)).browse(captor.capture());
  }

  private static void assertBetween(
      final OffsetDateTime actual,
      final OffsetDateTime lowerBound,
      final OffsetDateTime upperBound)
  {
    assertThat(lowerBound.compareTo(actual), lessThanOrEqualTo(0));
    assertThat(upperBound.compareTo(actual), greaterThanOrEqualTo(0));
  }
}
