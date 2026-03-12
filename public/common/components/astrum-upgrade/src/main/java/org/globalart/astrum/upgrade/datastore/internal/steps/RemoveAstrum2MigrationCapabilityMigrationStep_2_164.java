
package org.globalart.astrum.upgrade.datastore.internal.steps;

import java.sql.Connection;
import java.sql.PreparedStatement;
import java.util.Optional;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.springframework.stereotype.Component;

/**
 * Migration step which removes the defunct Astrum 2-to-3 migration capability.
 *
 * The migration capability was owned by the now-removed astrum-legacy-2-to-3-migration-plugin module.
 * Any instance that ever configured the wizard will have orphaned rows in capability_storage_item
 * that cause DefaultCapabilityRegistry to log "Cannot sync capability {} - unknown type {}" on
 * every sync.
 */
@Component
public class RemoveAstrum2MigrationCapabilityMigrationStep_2_164
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Override
  public Optional<String> version() {
    return Optional.of("2.164");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    if (tableExists(connection, "capability_storage_item")) {
      try (PreparedStatement statement =
          connection.prepareStatement("DELETE FROM capability_storage_item WHERE type = ?")) {
        statement.setString(1, "migration");
        int deleted = statement.executeUpdate();
        log.info("Removed {} orphaned Astrum 2-to-3 migration capability rows", deleted);
      }
    }
  }
}
