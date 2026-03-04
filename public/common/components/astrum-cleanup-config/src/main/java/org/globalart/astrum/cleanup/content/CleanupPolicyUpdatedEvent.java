
package org.globalart.astrum.cleanup.content;

import org.globalart.astrum.cleanup.storage.CleanupPolicy;

/**
 * {@link CleanupPolicyUpdated} event.
 *
 */
public class CleanupPolicyUpdatedEvent
    extends CleanupPolicyEvent
{

  public CleanupPolicyUpdatedEvent(final CleanupPolicy cleanupPolicy) {
    super(cleanupPolicy);
  }

}
