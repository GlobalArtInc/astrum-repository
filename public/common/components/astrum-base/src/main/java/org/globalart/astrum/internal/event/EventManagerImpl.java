
package org.globalart.astrum.internal.event;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.lifecycle.LifecycleSupport;
import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.event.EventAware.Asynchronous;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.common.event.HasAffinity;
import org.globalart.astrum.common.property.SystemPropertiesHelper;
import org.globalart.astrum.jmx.reflect.ManagedAttribute;
import org.globalart.astrum.jmx.reflect.ManagedObject;

import com.google.common.annotations.VisibleForTesting;
import com.google.common.eventbus.EventBus;
import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.EVENTS;
import static org.globalart.astrum.common.event.EventBusFactory.reentrantAsyncEventBus;
import static org.globalart.astrum.common.event.EventBusFactory.reentrantEventBus;

@Component
@ManagedLifecycle(phase = EVENTS)
// Start first within the EVENTS phase so the manager is available before AstrumEventAwareRegistrar registers
// subscribers; via the reversed shutdown order this also stops it last, after subscribers are unregistered.
@Order(Ordered.HIGHEST_PRECEDENCE)
@ManagedObject(typeClass = EventManager.class)
public class EventManagerImpl
    extends LifecycleSupport
    implements EventManager
{
  static final int HOST_THREAD_POOL_SIZE =
      SystemPropertiesHelper.getInteger("org.globalart.astrum.internal.event.EventManagerImpl.poolSize", 500);

  private final EventExecutor eventExecutor;

  private final EventBus eventBus;

  private final EventBus asyncBus;

  @Autowired
  public EventManagerImpl(final EventExecutor eventExecutor) {
    this.eventExecutor = checkNotNull(eventExecutor);

    this.eventBus = reentrantEventBus("astrum");
    this.asyncBus = reentrantAsyncEventBus("astrum.async", eventExecutor);
  }

  @Override
  public void register(final Object object) {
    boolean async = object instanceof Asynchronous;

    if (async) {
      asyncBus.register(object);
    }
    else {
      eventBus.register(object);
    }

    log.trace("Registered {}{}", async ? "ASYNC " : "", object);
  }

  @Override
  public void unregister(final Object object) {
    boolean async = object instanceof Asynchronous;

    if (async) {
      asyncBus.unregister(object);
    }
    else {
      eventBus.unregister(object);
    }

    log.trace("Unregistered {}{}", async ? "ASYNC " : "", object);
  }

  @Override
  public void post(final Object event) {
    // The EventManager is only usable once it has started (the EVENTS lifecycle phase). Posting earlier
    // (or after shutdown) is a lifecycle violation: the subscribers are not registered yet, so rather than
    // deliver to an incomplete bus we drop the event and warn, without failing the caller.
    if (!isStarted()) {
      log.warn("EventManager is not started; ignoring event posted outside the EVENTS phase: {}",
          event == null ? null : event.getClass().getName());
      return;
    }

    // notify synchronous subscribers before going asynchronous
    eventBus.post(event);

    if (isAffinityEnabled() && event instanceof HasAffinity) {
      String affinity = ((HasAffinity) event).getAffinity();
      if (affinity != null) {
        eventExecutor.executeWithAffinity(affinity, () -> asyncBus.post(event));
      }
      else {
        // unexpected state, fall back to previous behaviour
        log.warn("Event {} requested 'null' affinity", event);
        asyncBus.post(event);
      }
    }
    else {
      asyncBus.post(event);
    }
  }

  @Override
  @VisibleForTesting
  @ManagedAttribute
  public boolean isCalmPeriod() {
    return eventExecutor.isCalmPeriod();
  }

  @Override
  public boolean isAffinityEnabled() {
    return eventExecutor.isAffinityEnabled();
  }
}
