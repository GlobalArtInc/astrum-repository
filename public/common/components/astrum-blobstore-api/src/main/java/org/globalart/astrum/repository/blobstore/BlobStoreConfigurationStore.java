
package org.globalart.astrum.repository.blobstore;

import java.util.List;
import java.util.Optional;

import org.globalart.astrum.common.lifecycle.Lifecycle;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;

/**
 * {@link BlobStoreConfiguration} store.
 *
 * since 3.0
 */
public interface BlobStoreConfigurationStore
    extends Lifecycle
{
  /**
   * @return all BlobStoreConfigurations
   */
  List<BlobStoreConfiguration> list();

  /**
   * Persist a new BlobStoreConfiguration.
   */
  void create(BlobStoreConfiguration configuration);

  /**
   * Update an existing BlobStoreConfiguration.
   *
   * @since 3.14
   */
  void update(BlobStoreConfiguration configuration);

  /**
   * Delete an existing BlobStoreConfiguration.
   */
  void delete(BlobStoreConfiguration configuration);

  /**
   * Find a BlobStoreConfiguration by name.
   *
   * @since 3.14
   */
  BlobStoreConfiguration read(String name);

  /**
   * Find the parent group of a blob store
   *
   * @param name of the child to search on
   * @return the {@link Optional<BlobStoreConfiguration>} for the parent group if it exists
   *
   * @since 3.15
   */
  Optional<BlobStoreConfiguration> findParent(String name);

  /**
   * Create a new empty {@link BlobStoreConfiguration} suitable for use with this store
   *
   * @since 3.20
   */
  BlobStoreConfiguration newConfiguration();
}
