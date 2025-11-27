
package org.globalart.astrum.transaction;

import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;
import java.util.concurrent.ThreadLocalRandom;
import java.util.concurrent.TimeUnit;

import org.junit.Test;

import static java.util.concurrent.TimeUnit.MILLISECONDS;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

/**
 * Test rolling stats behaviour.
 */
public class RollingStatsTest

{
  @Test
  public void testConcurrentStats() throws Exception {
    RollingStats underTest = new RollingStats(60_000, MILLISECONDS);

    int iterations = 3;
    int threadCount = 100;
    ExecutorService executor = Executors.newFixedThreadPool(threadCount);
    List<Future<?>> futures = new ArrayList<>();
    for (int i = 0; i < iterations; i++) {
      for (int t = 0; t < threadCount; t++) {
        futures.add(executor.submit(() -> {
          // randomize where this will land inside the window
          Thread.sleep(ThreadLocalRandom.current().nextInt(1_000));
          underTest.mark();
          return null;
        }));
      }
    }
    executor.shutdown();
    executor.awaitTermination(60, TimeUnit.SECONDS);
    for (Future<?> f : futures) {
      f.get();
    }

    assertThat(underTest.sum(), is(threadCount * iterations));
  }

  @Test
  public void testRollingStats() throws Exception {
    RollingStats underTest = new RollingStats(2_000, MILLISECONDS);

    underTest.mark();
    assertThat(underTest.sum(), is(1));
    underTest.mark();
    assertThat(underTest.sum(), is(2));
    underTest.mark();
    assertThat(underTest.sum(), is(3));
    underTest.mark();
    assertThat(underTest.sum(), is(4));

    Thread.sleep(500);

    underTest.mark();
    assertThat(underTest.sum(), is(5));
    underTest.mark();
    assertThat(underTest.sum(), is(6));
    underTest.mark();
    assertThat(underTest.sum(), is(7));
    underTest.mark();
    assertThat(underTest.sum(), is(8));

    Thread.sleep(500);

    underTest.mark();
    assertThat(underTest.sum(), is(9));
    underTest.mark();
    assertThat(underTest.sum(), is(10));
    underTest.mark();
    assertThat(underTest.sum(), is(11));
    underTest.mark();
    assertThat(underTest.sum(), is(12));

    Thread.sleep(500);

    assertThat(underTest.sum(), is(12));

    Thread.sleep(750);

    assertThat(underTest.sum(), is(8));

    Thread.sleep(500);

    assertThat(underTest.sum(), is(4));

    Thread.sleep(500);

    assertThat(underTest.sum(), is(0));
  }
}
