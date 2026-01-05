
package org.globalart.astrum.security.user;

import org.globalart.astrum.common.event.EventManager;

/**
 * No-Op {@link EventManager} for use in tests.
 */
public class NoOpEventManager
    implements EventManager
{
  @Override
  public void register(final Object handler) {
  }

  @Override
  public void unregister(final Object handler) {
  }

  @Override
  public void post(final Object event) {
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
