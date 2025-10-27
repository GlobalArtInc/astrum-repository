
package org.globalart.astrum.common.event;

/**
 * Event mix-in that declares the affinity to use during asynchronous event delivery.
 *
 * This is a hint to try and maintain ordering between events with the same affinity
 * at the point they reach subscribers, even when event delivery is multi-threaded.
 *
 * @since 3.11
 */
public interface HasAffinity
{
  /**
   * @return affinity hint
   */
  String getAffinity();
}
