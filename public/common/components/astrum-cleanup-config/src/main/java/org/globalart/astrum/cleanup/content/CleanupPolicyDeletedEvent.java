
package org.globalart.astrum.cleanup.content;

import org.globalart.astrum.cleanup.storage.CleanupPolicy;

/**
 * {@link CleanupPolicyDeleted} event.
 *
 */
public class CleanupPolicyDeletedEvent
    extends CleanupPolicyEvent
{

  public CleanupPolicyDeletedEvent(final CleanupPolicy cleanupPolicy) {
    super(cleanupPolicy);
  }
}
