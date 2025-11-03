
package org.globalart.astrum.thread;

import java.util.concurrent.ThreadFactory;
import java.util.concurrent.atomic.AtomicInteger;

/**
 * Astrum {@link ThreadFactory}.
 */
public class AstrumThreadFactory
    implements ThreadFactory
{
  private static final AtomicInteger poolNumber = new AtomicInteger(1);

  private final AtomicInteger threadNumber = new AtomicInteger(1);

  private final String namePrefix;

  private final ThreadGroup schedulerThreadGroup;

  private final boolean deamonThread;

  private int threadPriority;

  public AstrumThreadFactory(String poolId, String threadGroupName) {
    this(poolId, threadGroupName, Thread.NORM_PRIORITY);
  }

  public AstrumThreadFactory(final String poolId, final String threadGroupName, final int threadPriority) {
    this(poolId, threadGroupName, threadPriority, false);
  }

  public AstrumThreadFactory(
      final String poolId,
      final String threadGroupName,
      final int threadPriority,
      final boolean daemonThread)
  {
    int poolNum = poolNumber.getAndIncrement();
    this.schedulerThreadGroup = new ThreadGroup(threadGroupName + " #" + poolNum);
    this.namePrefix = poolId + "-" + poolNum + "-thread-";
    this.deamonThread = daemonThread;
    this.threadPriority = threadPriority;
  }

  public Thread newThread(final Runnable r) {
    final Thread result = new Thread(schedulerThreadGroup, r, namePrefix + threadNumber.getAndIncrement());
    result.setDaemon(this.deamonThread);
    result.setPriority(this.threadPriority);
    return result;
  }
}
