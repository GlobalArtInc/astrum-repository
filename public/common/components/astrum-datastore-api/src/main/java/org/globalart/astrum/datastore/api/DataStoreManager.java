
package org.globalart.astrum.datastore.api;

import java.util.Optional;

import org.globalart.astrum.common.lifecycle.Lifecycle;
import org.globalart.astrum.common.app.Freezable;

/**
 * {@link DataStore} manager.
 *
 * @since 3.19
 */
public interface DataStoreManager
    extends Lifecycle, Freezable
{
  String DEFAULT_DATASTORE_NAME = "astrum";

  /**
   * Browse existing data stores.
   */
  Iterable<DataStore<?>> browse();

  /**
   * Create a new data store.
   */
  DataStore<?> create(DataStoreConfiguration configuration) throws Exception;

  /**
   * Update an existing data store.
   */
  DataStore<?> update(DataStoreConfiguration configuration) throws Exception;

  /**
   * Lookup a data store by name.
   */
  Optional<DataStore<?>> get(String storeName);

  /**
   * Delete a data store by name.
   */
  boolean delete(String storeName) throws Exception;

  /**
   * @return {@code true} if the named data store already exists. Check is case-insensitive.
   */
  boolean exists(String storeName);
}
