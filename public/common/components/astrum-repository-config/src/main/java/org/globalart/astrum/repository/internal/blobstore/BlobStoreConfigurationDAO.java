
package org.globalart.astrum.repository.internal.blobstore;

import java.util.Collection;

import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.datastore.api.IterableDataAccess;

/**
 * {@link BlobStoreConfigurationData} access.
 *
 * @since 3.21
 */
public interface BlobStoreConfigurationDAO
    extends IterableDataAccess.WithName<BlobStoreConfigurationData>
{
  /**
   * Find candidate configurations that might have the given name as a member.
   *
   * These candidates will then be filtered in the config store to find the exact match.
   * This way we can use a simple filter in the DB when JSON filtering is not available
   * without needing to have additional tables or denormalized attributes.
   */
  Collection<BlobStoreConfiguration> findCandidateParents(String name);
}
