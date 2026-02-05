
package org.globalart.astrum.repository.content.internal;

import java.sql.Connection;
import java.util.List;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.lifecycle.LifecycleSupport;
import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.datastore.api.DataSessionSupplier;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.UPGRADE;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;
import org.springframework.stereotype.Component;

/**
 * Checks database integrity at application startup and configures automated repairs
 */
@Component
@ManagedLifecycle(phase = UPGRADE)
public class DatabaseIntegrityCheckService
    extends LifecycleSupport
{
  private final DataSessionSupplier dataSessionSupplier;

  private final List<DatabaseIntegrityChecker> databaseIntegrityCheckers;

  @Autowired
  public DatabaseIntegrityCheckService(
      final DataSessionSupplier dataSessionSupplier,
      final List<DatabaseIntegrityChecker> databaseIntegrityCheckers)
  {
    this.dataSessionSupplier = checkNotNull(dataSessionSupplier);
    this.databaseIntegrityCheckers = checkNotNull(databaseIntegrityCheckers);
  }

  @Override
  protected void doStart() throws Exception {
    Connection connection = dataSessionSupplier.openConnection(DEFAULT_DATASTORE_NAME);
    for (DatabaseIntegrityChecker checker : databaseIntegrityCheckers) {
      checker.checkAndRepair(connection);
    }
  }
}
