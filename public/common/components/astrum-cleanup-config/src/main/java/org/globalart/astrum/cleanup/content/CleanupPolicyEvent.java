
package org.globalart.astrum.cleanup.content;

import org.globalart.astrum.cleanup.storage.CleanupPolicy;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * {@link CleanupPolicy} event.
 *
 */
public abstract class CleanupPolicyEvent
{
  private final CleanupPolicy cleanupPolicy;

  public CleanupPolicyEvent(final CleanupPolicy cleanupPolicy) {
    this.cleanupPolicy = checkNotNull(cleanupPolicy);
  }

  public CleanupPolicy getCleanupPolicy() {
    return cleanupPolicy;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "cleanupPolicy=" + cleanupPolicy +
        '}';
  }
}
