
package org.globalart.astrum.quartz.internal;

import java.util.concurrent.RejectedExecutionException;
import java.util.concurrent.Semaphore;
import java.util.concurrent.SynchronousQueue;
import java.util.concurrent.ThreadPoolExecutor;
import java.util.concurrent.ThreadPoolExecutor.AbortPolicy;
import java.util.concurrent.TimeUnit;

import org.globalart.astrum.security.subject.FakeAlmightySubject;
import org.globalart.astrum.thread.AstrumExecutorService;
import org.globalart.astrum.thread.AstrumThreadFactory;

import org.quartz.SchedulerConfigException;
import org.quartz.spi.ThreadPool;

import static com.google.common.base.Preconditions.checkArgument;

/**
 * Astrum specific implementation of {@link ThreadPool} that is integrated with Shiro.
 *
 * @since 3.0
 */
public class QuartzThreadPool
    implements ThreadPool
{
  /**
   * The "bare" executor (non-Shiro aware), needed to gather stats.
   */
  private final ThreadPoolExecutor threadPoolExecutor;

  /**
   * The shiro aware executor wrapper service.
   *
   * Using this wrapper are the Jobs scheduled and made Shiro Subject equipped.
   */
  private final AstrumExecutorService astrumExecutorService;

  /**
   * Used to block execution if thread pool is full.
   */
  private final Semaphore semaphore;

  private String instanceId;

  private String instanceName;

  public QuartzThreadPool(final int poolSize, final int threadPriority) {
    checkArgument(poolSize > 0, "Pool size must be greater than zero");
    checkArgument(threadPriority >= Thread.MIN_PRIORITY && threadPriority <= Thread.MAX_PRIORITY, String
        .format("Thread priority value must be an int between %s and %s", Thread.MIN_PRIORITY, Thread.MAX_PRIORITY));

    this.threadPoolExecutor = new ThreadPoolExecutor(
        poolSize, // core-size
        poolSize, // max-size
        0L, // keep-alive
        TimeUnit.MILLISECONDS,
        new SynchronousQueue<>(), // no queuing
        new AstrumThreadFactory("quartz", "as-tasks", threadPriority),
        new AbortPolicy());

    // wrapper for Shiro integration
    this.astrumExecutorService = AstrumExecutorService.forFixedSubject(
        threadPoolExecutor, FakeAlmightySubject.TASK_SUBJECT);

    semaphore = new Semaphore(poolSize);
  }

  @Override
  public void setInstanceId(final String instanceId) {
    this.instanceId = instanceId;
  }

  @Override
  public void setInstanceName(final String instanceName) {
    this.instanceName = instanceName;
  }

  @Override
  public int getPoolSize() {
    return threadPoolExecutor.getPoolSize();
  }

  //
  // Lifecycle
  //

  @Override
  public void initialize() throws SchedulerConfigException {
    // nop
  }

  @Override
  public void shutdown(final boolean waitForJobsToComplete) {
    astrumExecutorService.shutdown();
    if (waitForJobsToComplete) {
      try {
        astrumExecutorService.awaitTermination(5L, TimeUnit.SECONDS);
      }
      catch (InterruptedException e) {
        throw new RuntimeException(e);
      }
    }
  }

  //
  // Operations
  //

  @Override
  public boolean runInThread(final Runnable runnable) {
    try {
      semaphore.acquire();
      // this below is true as we do not use queue on executor combined with abort policy.
      // Meaning, if no exception, the task is accepted for execution
      // Use a wrapper to decrement the semaphore after the runnable completes
      astrumExecutorService.submit(semaphoreReleasingRunnable(runnable));
      return true;
    }
    catch (RejectedExecutionException | InterruptedException e) {
      // must decrement semaphore if job failed to submit
      semaphore.release();
      return false;
    }
  }

  /**
   * Wrap a runnable to release the semaphore after completion.
   */
  private Runnable semaphoreReleasingRunnable(final Runnable runnable) {
    return () -> {
      try {
        runnable.run();
      }
      finally {
        semaphore.release();
      }
    };
  }

  @Override
  public int blockForAvailableThreads() {
    try {
      semaphore.acquire();
      try {
        return semaphore.availablePermits() + 1;
      }
      finally {
        semaphore.release();
      }
    }
    catch (InterruptedException e) {
      throw new RuntimeException(e);
    }
  }
}
