
package org.globalart.astrum.scheduling.internal;

import java.time.Duration;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.ScheduledFuture;
import java.util.concurrent.TimeUnit;

import org.globalart.astrum.common.scheduling.PeriodicJobService;
import org.globalart.astrum.common.stateguard.Guarded;
import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;
import org.globalart.astrum.thread.AstrumThreadFactory;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkState;
import static org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport.State.STARTED;

/**
 * Default implementation of {@link PeriodicJobService}, based on a ScheduledExecutorService.
 *
 * @since 3.0
 */
@Component
public class PeriodicJobServiceImpl
    extends StateGuardLifecycleSupport
    implements PeriodicJobService
{
  private ScheduledExecutorService executor;

  private int activeClients;

  private int threadPoolSize;

  @Autowired
  public PeriodicJobServiceImpl(@Value("${astrum.periodicJob.threadPoolSize:10}") final int threadPoolSize) {
    String errorMsg =
        String.format("PeriodicJobServiceImpl's configurable threadPool must be a positive integer, (%d) is not valid.",
            threadPoolSize);
    checkArgument(threadPoolSize > 0, errorMsg);
    this.threadPoolSize = threadPoolSize;
  }

  @Override
  public synchronized void startUsing() {
    if (activeClients == 0) {
      try {
        start();
      }
      catch (Exception e) {
        throw new PeriodicJobStartException(e);
      }
    }
    activeClients++;
  }

  @Override
  public synchronized void stopUsing() {
    checkState(activeClients > 0, "Not started");
    activeClients--;
    if (activeClients == 0) {
      try {
        stop();
      }
      catch (Exception e) {
        throw new PeriodicJobShutdownException(e);
      }
    }
  }

  @Override
  protected void doStart() throws Exception {
    executor = Executors.newScheduledThreadPool(threadPoolSize, new AstrumThreadFactory("periodic", "scheduling"));
  }

  @Override
  protected void doStop() throws Exception {
    executor.shutdown();
    if (!executor.awaitTermination(10, TimeUnit.SECONDS)) {
      log.warn("Failed to terminate thread pool in allotted time");
    }
    executor = null;
  }

  @Override
  public void runOnce(final Runnable runnable, final int delaySeconds) {
    startUsing();
    executor.schedule(() -> {
      try {
        runnable.run();
        return null;
      }
      finally {
        stopUsing();
      }
    }, delaySeconds, TimeUnit.SECONDS);
  }

  @Override
  @Guarded(by = STARTED)
  public PeriodicJob schedule(final Runnable runnable, final int repeatPeriodSeconds) {
    ScheduledFuture<?> scheduledFuture = executor.scheduleAtFixedRate(
        wrap(runnable),
        repeatPeriodSeconds,
        repeatPeriodSeconds,
        TimeUnit.SECONDS);

    return () -> scheduledFuture.cancel(false);
  }

  @Override
  @Guarded(by = STARTED)
  public PeriodicJob schedule(final Runnable runnable, final Duration delay, final Duration repeatPeriod) {
    ScheduledFuture<?> scheduledFuture = executor.scheduleAtFixedRate(
        wrap(runnable),
        delay.toMillis(),
        repeatPeriod.toMillis(),
        TimeUnit.MILLISECONDS);

    return () -> scheduledFuture.cancel(false);
  }

  private Runnable wrap(final Runnable inner) {
    return () -> {
      try {
        inner.run();
      }
      catch (Exception e) {
        // Do not propagate as this will cancel the recurring job
        log.error("Periodic job threw exception", e);
      }
    };
  }

  public static class PeriodicJobShutdownException
      extends RuntimeException
  {
    private PeriodicJobShutdownException(final Exception e) {
      super(e);
    }
  }

  public static class PeriodicJobStartException
      extends RuntimeException
  {
    private PeriodicJobStartException(final Exception e) {
      super(e);
    }
  }
}
