
package org.globalart.astrum.common.event;

import com.google.common.eventbus.EventBus;

/**
 * Marker to indicate that component should be registered and unregistered with the synchronous {@link EventBus}.
 *
 * @since 3.0
 */
public interface EventAware
{
  /**
   * Marker for {@link EventAware} component to register and unregister with the asynchronous {@link EventBus}.
   *
   * <p>
   * This interface <b>extends</b> {@link EventAware}, so declaring {@code implements EventAware.Asynchronous}
   * alone is sufficient: the component is discovered as an {@link EventAware} subscriber and routed to the
   * asynchronous bus. Declaring both {@code EventAware, EventAware.Asynchronous} is equivalent and harmless.
   *
   * @apiNote Making {@code Asynchronous} extend {@link EventAware} closes the trap behind ASTRUM-52911
   *          (June 2026): a class that kept only the {@code Asynchronous} marker after a refactor previously
   *          compiled cleanly but silently skipped registration, and all webhook dispatch went dark for
   *          ~3 weeks (fixed under ASTRUM-53667). With the inheritance in place that failure mode is no
   *          longer possible.
   *          <p>
   *          Note that only <b>singleton</b> beans are auto-registered by {@code AstrumEventAwareRegistrar};
   *          per-repository (prototype-scoped) components such as repository {@code Facet}s register and
   *          unregister themselves via {@code FacetSupport} as their repository starts and stops.
   */
  interface Asynchronous
      extends EventAware
  {
    // empty
  }
}
