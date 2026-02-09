
package org.globalart.astrum.repository.content.upgrades;

import java.sql.Connection;
import java.sql.Statement;
import java.util.List;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Updates key-value tables in Postgres to change the value column to TEXT from VARCHAR
 */
@Component
public class KeyValueMigrationStep_2_78
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Override
  public Optional<String> version() {
    return Optional.of("2.78");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    if (isPostgresql(connection)) {
      try (Statement statment = connection.createStatement()) {
        for (String format : List.of("apt", "helm", "yum")) {
          log.info("Updating {}_key_value type", format);
          statment.execute("""
              ALTER TABLE IF EXISTS %s_key_value
              ALTER COLUMN value TYPE text;""".formatted(format));
        }
      }
    }
  }
}
