
package org.globalart.astrum.common.event;

/**
 * Interface exposing Guava {@link com.google.common.eventbus.EventBus} API.
 *
 * @since 3.0
 *
 * @deprecated use {@link EventManager} instead
 */
@Deprecated
public interface EventBus
{
  /**
   * Registers an event handler with this event bus.
   *
   * @param handler to be registered
   */
  void register(Object handler);

  /**
   * Unregisters an event handler from this event bus.
   *
   * @param handler to be unregistered
   */
  void unregister(Object handler);

  /**
   * Posts an event. Event bus will notify all previously registered handlers about this event.
   *
   * @param event an event
   */
  void post(Object event);
}
