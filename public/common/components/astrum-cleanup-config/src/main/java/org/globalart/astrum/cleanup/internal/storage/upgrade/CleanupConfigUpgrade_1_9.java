
package org.globalart.astrum.cleanup.internal.storage.upgrade;

import java.sql.Connection;
import java.sql.Statement;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

@Component
public class CleanupConfigUpgrade_1_9
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Override
  public Optional<String> version() {
    return Optional.of("1.9");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {

    try (Statement stmt = connection.createStatement()) {
      int updates = stmt.executeUpdate("UPDATE CLEANUP_POLICY " +
          " SET FORMAT='ALL_FORMATS' WHERE FORMAT='*'");
      log.info(
          "Updated {} misconfigured cleanup policies: set format='ALL_FORMATS'",
          updates);
    }
  }
}
