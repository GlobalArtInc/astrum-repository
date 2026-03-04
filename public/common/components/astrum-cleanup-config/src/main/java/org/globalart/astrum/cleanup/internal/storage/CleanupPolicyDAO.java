
package org.globalart.astrum.cleanup.internal.storage;

import org.globalart.astrum.cleanup.storage.CleanupPolicy;
import org.globalart.astrum.datastore.api.NamedDataAccess;

/**
 * {@link CleanupPolicyData} access.
 *
 * @since 3.21
 */
public interface CleanupPolicyDAO
    extends NamedDataAccess<CleanupPolicyData>
{
  /**
   * Current count of persisted {@link CleanupPolicy}'s.
   */
  int count();

  /**
   * Get {@link CleanupPolicy}'s for a specific format.
   */
  Iterable<CleanupPolicy> browseByFormat(String format);
}
