
package org.globalart.astrum.testcommon.event;

import org.globalart.astrum.common.event.EventManager;

import com.google.common.annotations.VisibleForTesting;
import com.google.common.eventbus.EventBus;

import static org.globalart.astrum.common.event.EventBusFactory.reentrantEventBus;

/**
 * Simple {@link EventManager} for UT purposes. Requires manual registration, doesn't support asynchronous dispatch.
 *
 * @since 3.2
 */
@VisibleForTesting
public class SimpleEventManager
    implements EventManager
{
  private final EventBus eventBus = reentrantEventBus("simple");

  @Override
  public void register(final Object handler) {
    eventBus.register(handler);
  }

  @Override
  public void unregister(final Object handler) {
    eventBus.unregister(handler);
  }

  @Override
  public void post(final Object event) {
    eventBus.post(event);
  }

  @Override
  public boolean isCalmPeriod() {
    return true;
  }

  @Override
  public boolean isAffinityEnabled() {
    return false;
  }
}
