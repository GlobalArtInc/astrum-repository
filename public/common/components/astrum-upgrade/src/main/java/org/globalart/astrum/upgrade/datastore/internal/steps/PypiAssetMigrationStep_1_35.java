
package org.globalart.astrum.upgrade.datastore.internal.steps;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Drops all cached metadata in pypi group repositories, will be regenerated on request and stored in proper location
 */
@Component
public class PypiAssetMigrationStep_1_35
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String DROP_ASSETS = "DELETE FROM pypi_asset WHERE repository_id IN(" +
      "SELECT cr.repository_id FROM " +
      "pypi_content_repository cr " +
      "LEFT JOIN repository AS r ON cr.config_repository_id = r.id " +
      "WHERE r.recipe_name = 'pypi-group')";

  @Override
  public Optional<String> version() {
    return Optional.of("1.35");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    if (tableExists(connection, "pypi_asset") && tableExists(connection, "pypi_content_repository")) {
      log.info("Removing cached assets in pypi group repositories, regenerated with proper storage path on request.");
      runStatement(connection, DROP_ASSETS);
    }
  }
}
