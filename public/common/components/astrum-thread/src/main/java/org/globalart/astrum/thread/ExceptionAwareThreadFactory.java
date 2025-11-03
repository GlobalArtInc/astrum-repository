
package org.globalart.astrum.thread;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class ExceptionAwareThreadFactory
    extends AstrumThreadFactory
{

  private static final Logger log = LoggerFactory.getLogger(ExceptionAwareThreadFactory.class);

  public ExceptionAwareThreadFactory(final String poolId, final String threadGroupName) {
    super(poolId, threadGroupName);
  }

  public ExceptionAwareThreadFactory(final String poolId, final String threadGroupName, final int threadPriority) {
    super(poolId, threadGroupName, threadPriority);
  }

  public ExceptionAwareThreadFactory(
      final String poolId,
      final String threadGroupName,
      final int threadPriority,
      final boolean daemonThread)
  {
    super(poolId, threadGroupName, threadPriority, daemonThread);
  }

  @Override
  public Thread newThread(final Runnable r) {
    Thread tr = super.newThread(r);
    tr.setUncaughtExceptionHandler((t, e) -> {
      log.error("Uncaught Exception occurred on thread: {}, Exception message: {}", t.getName(), e.getMessage());
    });
    return tr;
  }
}
