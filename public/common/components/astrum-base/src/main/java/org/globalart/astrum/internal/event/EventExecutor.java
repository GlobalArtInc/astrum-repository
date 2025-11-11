
package org.globalart.astrum.internal.event;

import java.time.Duration;
import java.util.concurrent.Executor;
import java.util.concurrent.LinkedBlockingQueue;
import java.util.concurrent.RejectedExecutionHandler;
import java.util.concurrent.SynchronousQueue;
import java.util.concurrent.ThreadPoolExecutor;
import java.util.concurrent.ThreadPoolExecutor.CallerRunsPolicy;
import java.util.concurrent.TimeUnit;
import java.util.function.Supplier;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.time.Time;
import org.globalart.astrum.common.lifecycle.LifecycleSupport;
import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.event.EventAware.Asynchronous;
import org.globalart.astrum.common.event.HasAffinity;
import org.globalart.astrum.thread.AstrumExecutorService;
import org.globalart.astrum.thread.AstrumThreadFactory;

import com.google.common.annotations.VisibleForTesting;
import com.google.common.cache.CacheBuilder;
import com.google.common.cache.CacheLoader;
import com.google.common.cache.LoadingCache;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Preconditions.checkState;
import static com.google.common.util.concurrent.MoreExecutors.newSequentialExecutor;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.TASKS;
import static org.globalart.astrum.common.event.EventHelper.asReplicating;
import static org.globalart.astrum.common.event.EventHelper.isReplicating;
import static org.globalart.astrum.internal.event.EventManagerImpl.HOST_THREAD_POOL_SIZE;

/**
 * Custom {@link Executor} used to dispatch events to {@link Asynchronous} subscribers.
 *
 * As Astrum starts, subscribers are called directly by the originating thread. Once the
 * TASKS phase is reached subscribers will be called asynchronously using a thread pool.
 *
 * Conversely as Astrum stops, the thread pool is shutdown after leaving the TASKS phase
 * and subscribers will again be called directly by the originating thread. This avoids
 * asynchronous subscribers from having services disappear beneath them.
 *
 * @since 3.2
 */
@Component
@ManagedLifecycle(phase = TASKS)
public class EventExecutor
    extends LifecycleSupport
    implements Executor
{
  /**
   * Like {@link CallerRunsPolicy} but it continues to work after the executor is shutdown.
   */
  private static final RejectedExecutionHandler CALLER_RUNS_FAILSAFE = (command, executor) -> command.run();

  private final boolean affinityEnabled;

  private final int affinityCacheSize;

  private final Time affinityTimeout;

  private final boolean singleCoordinator;

  private final boolean fairThreading;

  private AstrumExecutorService eventProcessor;

  private AstrumExecutorService affinityProcessor;

  private LoadingCache<String, AffinityBarrier> affinityBarriers;

  private volatile boolean asyncProcessing;

  @Autowired
  public EventExecutor(
      @Value("${astrum.event.affinityEnabled:true}") final boolean affinityEnabled,
      @Value("${astrum.event.affinityCacheSize:1000}") final int affinityCacheSize,
      @Value("${astrum.event.affinityTimeout:1s}") final Duration affinityTimeout,
      @Value("${astrum.event.singleCoordinator:false}") final boolean singleCoordinator,
      @Value("${astrum.event.fairThreading:false}") final boolean fairThreading)
  {
    this.affinityEnabled = affinityEnabled;
    this.affinityCacheSize = affinityCacheSize;
    this.affinityTimeout = Time.seconds(checkNotNull(affinityTimeout).getSeconds());
    this.singleCoordinator = singleCoordinator;
    this.fairThreading = fairThreading;
  }

  /**
   * Move from direct to asynchronous subscriber processing.
   */
  @Override
  protected void doStart() throws Exception {

    ThreadPoolExecutor threadPool = new ThreadPoolExecutor(
        0,
        HOST_THREAD_POOL_SIZE,
        60L,
        TimeUnit.SECONDS,
        new SynchronousQueue<>(fairThreading), // rendezvous only, zero capacity
        new AstrumThreadFactory("event", "event-manager"),
        CALLER_RUNS_FAILSAFE);

    eventProcessor = AstrumExecutorService.forCurrentSubject(threadPool);

    if (affinityEnabled) {

      Supplier<Executor> coordinator;
      if (singleCoordinator) {
        // like singleThreadExecutor but with custom rejection handler
        ThreadPoolExecutor affinityThread = new ThreadPoolExecutor(
            1,
            1,
            0L,
            TimeUnit.SECONDS,
            new LinkedBlockingQueue<>(), // allow queueing up of requests
            new AstrumThreadFactory("affinity", "affinity-manager"),
            CALLER_RUNS_FAILSAFE);

        affinityProcessor = AstrumExecutorService.forCurrentSubject(affinityThread);

        // use single-thread queue to coordinate all events (delivery to subscribers is still multi-threaded)
        coordinator = () -> affinityProcessor;
      }
      else {
        // multi-threaded coordination and delivery, with sequential coordination for events with same affinity
        coordinator = () -> newSequentialExecutor(eventProcessor); // wraps behaviour on top of eventProcessor
      }

      affinityBarriers = CacheBuilder.newBuilder()
          .maximumSize(affinityCacheSize)
          .build(CacheLoader.from(() -> new AffinityBarrier(coordinator.get(), eventProcessor, affinityTimeout)));
    }

    asyncProcessing = true;
  }

  @Override
  protected void doStop() throws Exception {
    if (asyncProcessing) {
      shutdown(affinityProcessor);
      shutdown(eventProcessor);
      asyncProcessing = false;
    }
  }

  /**
   * Used by UTs and ITs only, to "wait for calm period", when all the async event subscribers finished.
   */
  @VisibleForTesting
  boolean isCalmPeriod() {
    if (asyncProcessing) {
      return isCalmPeriod(affinityProcessor) && isCalmPeriod(eventProcessor);
    }
    else {
      return true; // single-threaded mode is always calm
    }
  }

  /**
   * Is {@link HasAffinity} support enabled?
   *
   * @since 3.11
   */
  public boolean isAffinityEnabled() {
    return affinityEnabled;
  }

  /**
   * Executes asynchronous posting of an event using affinity to maintain event ordering across threads.
   *
   * @since 3.11
   */
  public void executeWithAffinity(final String affinity, final Runnable postEventToAsyncBus) {
    checkState(affinityEnabled);
    if (asyncProcessing) {
      Runnable command = inheritIsReplicating(postEventToAsyncBus);
      AffinityBarrier barrier = affinityBarriers.getUnchecked(affinity);
      barrier.coordinate(command); // waits (on separate thread) for last event delivery to complete before posting
    }
    else {
      postEventToAsyncBus.run();
    }
  }

  /**
   * Executes asynchronous delivery of an event to a particular subscriber, tracking it as necessary.
   */
  @Override
  public void execute(final Runnable deliverEventToSubscriber) {
    if (asyncProcessing) {
      Runnable command = inheritIsReplicating(deliverEventToSubscriber);
      AffinityBarrier barrier = affinityEnabled ? AffinityBarrier.current() : null;
      if (barrier != null) {
        barrier.execute(command); // tracks each event delivery to help with coordination of the next posting request
      }
      else {
        eventProcessor.execute(command);
      }
    }
    else {
      deliverEventToSubscriber.run();
    }
  }

  /**
   * @return {@code true} if the thread pool backing the (optional) executor service is inactive
   */
  private boolean isCalmPeriod(@Nullable final AstrumExecutorService executorService) {
    if (executorService != null) {
      ThreadPoolExecutor threadPool = (ThreadPoolExecutor) executorService.getTargetExecutorService();
      return threadPool.getQueue().isEmpty() && threadPool.getActiveCount() == 0;
    }
    else {
      return true; // service not enabled, consider as calm
    }
  }

  /**
   * @return {@code true} if the thread pool backing the (optional) executor service is shutdown
   */
  private void shutdown(@Nullable final AstrumExecutorService executorService) {
    if (executorService != null) {
      ThreadPoolExecutor threadPool = (ThreadPoolExecutor) executorService.getTargetExecutorService();
      threadPool.shutdown();
      try {
        threadPool.awaitTermination(5L, TimeUnit.SECONDS);
      }
      catch (InterruptedException e) {
        log.debug("Interrupted while waiting for termination", e);
      }
    }
  }

  /**
   * Binds current "isReplicating" context to the {@link Runnable} regardless which thread executes it.
   */
  private static Runnable inheritIsReplicating(final Runnable command) {
    if (!isReplicating()) {
      return command; // no need to inherit flag
    }
    return () -> {
      // check state from context of thread doing the running
      if (!isReplicating()) {
        asReplicating(command); // set flag for duration of command
      }
      else {
        command.run(); // flag already set, maintain it
      }
    };
  }
}
