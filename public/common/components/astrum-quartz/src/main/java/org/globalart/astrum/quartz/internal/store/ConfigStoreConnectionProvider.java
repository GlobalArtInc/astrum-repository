
package org.globalart.astrum.quartz.internal.store;

import java.sql.Connection;
import java.sql.SQLException;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.datastore.api.DataStore;

import org.quartz.utils.ConnectionProvider;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;
import org.springframework.stereotype.Component;

/**
 * Supplies JDBC connections to Quartz from the shared config {@link DataStore}.
 *
 * @since 3.19
 */
@Component
public class ConfigStoreConnectionProvider
    implements ConnectionProvider
{
  private final DataSessionSupplier sessionSupplier;

  @Autowired
  public ConfigStoreConnectionProvider(final DataSessionSupplier sessionSupplier) {
    this.sessionSupplier = checkNotNull(sessionSupplier);
  }

  @Override
  public void initialize() {
    // no-op
  }

  @Override
  public Connection getConnection() throws SQLException {
    return sessionSupplier.openConnection(DEFAULT_DATASTORE_NAME);
  }

  @Override
  public void shutdown() {
    // no-op
  }
}
