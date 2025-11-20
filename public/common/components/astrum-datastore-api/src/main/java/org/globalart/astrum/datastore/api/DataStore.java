
package org.globalart.astrum.datastore.api;

import java.sql.Connection;
import java.sql.SQLException;
import javax.sql.DataSource;

import org.globalart.astrum.common.lifecycle.Lifecycle;
import org.globalart.astrum.transaction.TransactionalStore;

/**
 * Each {@link DataStore} contains a number of {@link DataAccess} mappings accessible via {@link DataSession}s.
 *
 * @since 3.19
 */
public interface DataStore<S extends DataSession<?>>
    extends TransactionalStore<S>, Lifecycle
{
  /**
   * Configure the data store; changes won't take effect until the store is (re)started.
   */
  void setConfiguration(DataStoreConfiguration configuration);

  /**
   * @return the data store's configuration.
   */
  DataStoreConfiguration getConfiguration();

  /**
   * @return {@code true} if the data store has been started.
   */
  boolean isStarted();

  /**
   * Registers the given {@link DataAccess} type with this store.
   */
  void register(Class<? extends DataAccess> accessType);

  /**
   * Unregisters the given {@link DataAccess} type with this store.
   */
  void unregister(Class<? extends DataAccess> accessType);

  /**
   * Opens a new JDBC {@link Connection} to this store.
   *
   * @throws UnsupportedOperationException if this store doesn't support JDBC
   */
  Connection openConnection() throws SQLException;

  /**
   * @since 3.29
   */
  DataSource getDataSource();

  /**
   * Permanently stops this data store.
   */
  void shutdown() throws Exception;

  /**
   * Backup this data store to the specified location.
   *
   * @throws UnsupportedOperationException if the underlying data store does not support backing up
   *
   * @since 3.21
   */
  void backup(String location) throws Exception;
}
