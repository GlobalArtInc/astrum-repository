
package org.globalart.astrum.thread;

import java.util.concurrent.ForkJoinPool;

/**
 * @since 3.20
 */
public class AstrumForkJoinPoolFactory
{
  private AstrumForkJoinPoolFactory() {
    // static utility class
  }

  // Copied from the constant in {@link ForkJoinPool} because the constant is not visible.
  private static final int MAX_CAP = 0x7fff;

  public static ForkJoinPool createForkJoinPool(final String threadNamePrefix) {
    AstrumForkJoinWorkerThreadFactory astrumForkJoinWorkerThreadFactory =
        new AstrumForkJoinWorkerThreadFactory(threadNamePrefix);
    return new ForkJoinPool(Math.min(MAX_CAP, Runtime.getRuntime().availableProcessors()),
        astrumForkJoinWorkerThreadFactory, null, false);
  }
}
