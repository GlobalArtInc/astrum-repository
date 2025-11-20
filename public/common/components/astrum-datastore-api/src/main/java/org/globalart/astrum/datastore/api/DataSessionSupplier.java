
package org.globalart.astrum.datastore.api;

import java.sql.Connection;
import java.sql.SQLException;
import java.util.Optional;

import javax.sql.DataSource;

/**
 * {@link DataSession} supplier; for use by clients who don't need the full store API.
 *
 * @since 3.19
 */
public interface DataSessionSupplier
{
  /**
   * Opens a new {@link DataSession} against the named data store.
   *
   * @throws DataStoreNotFoundException if the store does not exist
   */
  DataSession<?> openSession(String storeName);

  /**
   * Opens a new {@link DataSession} against the named data store and sets the transaction isolation to Serializable.
   *
   * @throws DataStoreNotFoundException if the store does not exist
   * @since 3.38
   */
  DataSession<?> openSerializableTransactionSession(String storeName);

  /**
   * Opens a new JDBC {@link Connection} to the named data store.
   *
   * @throws DataStoreNotFoundException if the store does not exist
   * @throws UnsupportedOperationException if the store doesn't support JDBC
   */
  Connection openConnection(String storeName) throws SQLException;

  default Optional<DataSource> getDataSource(final String storeName) {
    throw new UnsupportedOperationException();
  }
}
