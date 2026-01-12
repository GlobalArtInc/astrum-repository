
package org.globalart.astrum.capability;

import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.TimeUnit;

import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.app.ManagedLifecycle.Phase;
import org.globalart.astrum.common.event.EventHelper;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.common.stateguard.Guarded;
import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;
import org.globalart.astrum.thread.AstrumExecutorService;
import org.globalart.astrum.thread.AstrumThreadFactory;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport.State.STARTED;

/**
 * An event broadcasting queue for Capability state change events. These are handled asynchronously to avoid the
 * capability registry from holding a lock while event handlers are performing other actions.
 */
@ManagedLifecycle(phase = Phase.CAPABILITIES)
@Order(Ordered.HIGHEST_PRECEDENCE)
@Component
public class CapabilityEventQueue
    extends StateGuardLifecycleSupport
{
  private final EventManager eventManager;

  private AstrumExecutorService eventQueue;

  @Autowired
  public CapabilityEventQueue(final EventManager eventManager) {
    this.eventManager = checkNotNull(eventManager);
  }

  @Override
  protected void doStart() {
    ExecutorService executor = Executors.newSingleThreadExecutor(
        new AstrumThreadFactory("capability-event", "capability-event"));
    // Wrap with AstrumExecutorService to preserve Shiro Subject (security context) across threads
    eventQueue = AstrumExecutorService.forCurrentSubject(executor);
  }

  @Override
  protected void doStop() throws Exception {
    if (eventQueue != null) {
      eventQueue.shutdown();
      try {
        eventQueue.awaitTermination(1, TimeUnit.MINUTES);
      }
      catch (Exception e) {
        eventQueue.shutdownNow();
        if (e instanceof InterruptedException) {
          Thread.currentThread().interrupt();
        }
        throw e;
      }
    }
  }

  @Guarded(by = STARTED)
  public void post(final CapabilityEvent event) {
    if (EventHelper.isReplicating()) {
      eventQueue.submit(() -> EventHelper.asReplicating(() -> eventManager.post(event)));
    }
    else {
      eventQueue.submit(() -> eventManager.post(event));
    }
  }
}
