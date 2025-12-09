
package org.globalart.astrum.internal.capability.storage.datastore.cleanup;

import java.sql.Connection;
import java.util.Optional;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Cleanup duplicate capabilities migration step. - do cleanup capabilities duplicates (if they exist) - create unique
 * index on capability_storage_item table
 */
@Component
public class CleanupCapabilityDuplicatesMigrationStep_1_27
    implements DatabaseMigrationStep
{
  private static final String ADD_INDEX =
      "CREATE UNIQUE INDEX IF NOT EXISTS uk_capability_storage_item_type_props ON capability_storage_item(type, properties)";

  private static final String ADD_CONSTRAINT =
      "ALTER TABLE capability_storage_item ADD CONSTRAINT IF NOT EXISTS uk_capability_storage_item_type_props UNIQUE (type, properties)";

  private final CleanupCapabilityDuplicatesService cleanupService;

  @Autowired
  public CleanupCapabilityDuplicatesMigrationStep_1_27(final CleanupCapabilityDuplicatesService cleanupService) {
    this.cleanupService = checkNotNull(cleanupService);
  }

  @Override
  public Optional<String> version() {
    return Optional.of("1.27");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    cleanupService.doCleanup();

    if (isPostgresql(connection)) {
      runStatement(connection, ADD_INDEX);
    }
    else {
      runStatement(connection, ADD_CONSTRAINT);
    }
  }
}
