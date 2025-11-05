
package org.globalart.astrum.thread;

import java.util.concurrent.Future;

import org.junit.Before;
import org.junit.Test;

import static org.awaitility.Awaitility.await;
import static java.util.concurrent.TimeUnit.MILLISECONDS;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.core.Is.is;

public class DatabaseStatusDelayedExecutorTest
{
  private static final int THREAD_POOL_SIZE = 1;

  private DatabaseStatusDelayedExecutor statusDelayedExecutor;

  @Before
  public void setup() throws Exception {
    statusDelayedExecutor = new DatabaseStatusDelayedExecutor(THREAD_POOL_SIZE);
    statusDelayedExecutor.start();
  }

  @Test
  public void ensureThatTaskRuns() throws Exception {
    Future<String> result = statusDelayedExecutor.submit(() -> "Done");

    await()
        .atMost(1000, MILLISECONDS)
        .until(() -> result.isDone());

    assertThat(result.get(), is("Done"));
  }

  @Test
  public void tasksExecuteImmediately() throws Exception {
    Future<String> result = statusDelayedExecutor.submit(() -> "Completed");

    await()
        .atMost(500, MILLISECONDS)
        .until(result::isDone);

    assertThat(result.get(), is("Completed"));
  }
}
