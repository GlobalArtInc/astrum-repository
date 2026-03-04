
package org.globalart.astrum.cleanup.content;

import org.globalart.astrum.cleanup.storage.CleanupPolicy;

/**
 * {@link CleanupPolicyCreated} event.
 *
 */
public class CleanupPolicyCreatedEvent
    extends CleanupPolicyEvent
{

  public CleanupPolicyCreatedEvent(final CleanupPolicy cleanupPolicy) {
    super(cleanupPolicy);
  }
}
