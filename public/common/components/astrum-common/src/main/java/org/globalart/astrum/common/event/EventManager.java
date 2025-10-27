
package org.globalart.astrum.common.event;

import com.google.common.annotations.VisibleForTesting;

/**
 * Event manager.
 *
 * @see EventAware
 * @since 3.0
 */
@SuppressWarnings("deprecation")
public interface EventManager
    extends EventBus
{
  /**
   * Registers an event handler with the event manager.
   *
   * @param handler to be registered
   *
   * @since 3.2
   */
  @Override
  void register(Object handler);

  /**
   * Unregisters an event handler from the event manager.
   *
   * @param handler to be unregistered
   *
   * @since 3.2
   */
  @Override
  void unregister(Object handler);

  /**
   * Posts an event, notifying all registered handlers.
   * <p>
   * The event manager only delivers events once it has started (the {@code EVENTS} lifecycle phase) and its
   * subscribers have been registered. Events posted before it has started, or after it has been shut down, are
   * dropped with a warning rather than delivered to an incompletely-wired bus; the caller is not failed.
   *
   * @param event an event
   *
   * @since 3.2
   */
  @Override
  void post(Object event);

  /**
   * Used by UTs and ITs only to "wait for calm period" when all async event handlers have finished.
   */
  @VisibleForTesting
  boolean isCalmPeriod();

  /**
   * Is {@link HasAffinity} support enabled?
   *
   * @since 3.11
   */
  boolean isAffinityEnabled();
}
