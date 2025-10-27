
package org.globalart.astrum.common.event;

/**
 * Event mix-in to determine whether or not an event is local in a distributed configuration.
 *
 * @since 3.21
 */
public interface HasLocality
    extends Event
{
  /**
   * @return {@code true} if this is a local event
   */
  boolean isLocal();
}
