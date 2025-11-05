
package org.globalart.astrum.thread;

import java.util.concurrent.ForkJoinPool;
import java.util.concurrent.ForkJoinWorkerThread;

import org.hamcrest.CoreMatchers;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;

/**
 * @since 3.20
 */
public class AstrumForkJoinPoolFactoryTest
{
  @Test
  public void threadsHaveCustomPrefix() {
    ForkJoinPool forkJoinPool = AstrumForkJoinPoolFactory.createForkJoinPool("custom-prefix-test-");
    ForkJoinWorkerThread thread = forkJoinPool.getFactory().newThread(forkJoinPool);
    assertThat(thread.getName(), CoreMatchers.containsString("custom-prefix-test-"));
  }

}
