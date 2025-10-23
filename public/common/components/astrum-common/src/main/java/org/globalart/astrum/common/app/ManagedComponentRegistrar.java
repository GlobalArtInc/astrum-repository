
package org.globalart.astrum.common.app;

/**
 * Call-back invoked by the lifecycle manager as {@link ManagedLifecycle} components move through their phases, so
 * cross-cutting concerns can be wired to a component <em>at the moment it becomes (or stops being) active</em> rather
 * than up-front.
 *
 * <p>
 * The lifecycle manager discovers every {@code ManagedComponentRegistrar} bean and notifies each one:
 * <ul>
 * <li>{@link #onStarted(Object)} &mdash; immediately <b>after</b> a component's {@code start()} completes
 * successfully;</li>
 * <li>{@link #onStopping(Object)} &mdash; immediately <b>before</b> a component's {@code stop()} is invoked.</li>
 * </ul>
 *
 * <p>
 * This gives registrars phase-accurate timing: a component managed in a late phase is handed to registrars only once
 * it has actually started, and is withdrawn before it stops. The canonical implementation registers
 * {@link org.globalart.astrum.common.event.EventAware} singletons with the event bus, so a subscriber can never receive
 * an event before its owning component has started &mdash; without each subscriber needing to guard its handlers.
 *
 * <p>
 * Implementations must be tolerant of components they do not care about (typically via an {@code instanceof} check)
 * and must be idempotent: the manager only notifies once per start/stop transition, but callbacks may arrive for a
 * component before the registrar itself has started (e.g. components in earlier phases). Neither method should throw;
 * the manager logs and continues if one does, so a misbehaving registrar cannot break startup or shutdown.
 *
 * @since 3.next
 */
public interface ManagedComponentRegistrar
{
  /**
   * Invoked immediately after {@code component.start()} completes successfully.
   */
  void onStarted(Object component);

  /**
   * Invoked immediately before {@code component.stop()} is invoked.
   */
  void onStopping(Object component);
}
