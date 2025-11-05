
package org.globalart.astrum.thread;

import java.util.concurrent.ForkJoinPool;
import java.util.concurrent.ForkJoinWorkerThread;

import org.hamcrest.CoreMatchers;
import org.junit.Test;
import static org.hamcrest.MatcherAssert.assertThat;

/**
 * @since 3.20
 */
public class AstrumForkJoinWorkerThreadFactoryTest
{
  private AstrumForkJoinWorkerThreadFactory astrumForkJoinWorkerThreadFactory;

  @Test
  public void prefixIsAddedToThread() {
    astrumForkJoinWorkerThreadFactory = new AstrumForkJoinWorkerThreadFactory("prefix-test");
    ForkJoinPool forkJoinPool = new ForkJoinPool();
    ForkJoinWorkerThread thread = astrumForkJoinWorkerThreadFactory.newThread(forkJoinPool);
    assertThat(thread.getName(), CoreMatchers.containsString("prefix-test"));
  }

}
