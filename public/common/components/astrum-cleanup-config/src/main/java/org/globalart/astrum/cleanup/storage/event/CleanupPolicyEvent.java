
package org.globalart.astrum.cleanup.storage.event;

import org.globalart.astrum.cleanup.storage.CleanupPolicy;

/**
 * Base interface for {@link CleanupPolicy} storage events.
 *
 * @since 3.14
 */
public interface CleanupPolicyEvent
{
  /**
   * Get the {@link CleanupPolicy} associated with the event.
   */
  CleanupPolicy getCleanupPolicy();

  /**
   * True if this event originated with the local node.
   */
  boolean isLocal();
}
