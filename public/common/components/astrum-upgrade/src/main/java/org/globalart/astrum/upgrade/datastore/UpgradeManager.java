
package org.globalart.astrum.upgrade.datastore;

import java.util.Collection;
import java.util.Collections;
import java.util.Optional;

import javax.annotation.Nullable;

import org.flywaydb.core.api.MigrationVersion;

/**
 * Manages database upgrades for SQL
 */
public interface UpgradeManager
{
  /**
   * Trigger database migration as the system user.
   *
   * @throws UpgradeException when a failure occurs
   */
  default void migrate() throws UpgradeException {
    migrate(null, Collections.emptyList());
  }

  /**
   * Trigger database migration with a specified user.
   *
   * @param user the user who requested database migration occur
   * @throws UpgradeException when a failure occurs
   */
  void migrate(@Nullable String user, final Collection<String> nodeIds) throws UpgradeException;

  /**
   * Validates that the current database schema version is supported by the running Astrum.
   *
   * @throws UpgradeException when the current schema version is not acceptable
   */
  void checkSchemaVersionIsAcceptable() throws UpgradeException;

  /**
   * @return {@code true} when there are pending migrations, {@code false} otherwise. For single node instances this
   *         should always be false after startup.
   */
  boolean requiresMigration();

  /**
   * Get the current database schema version
   */
  Optional<MigrationVersion> getCurrentVersion();

  /**
   * Get the latest available schema version known
   */
  Optional<MigrationVersion> getMaxMigrationVersion();

  void checkBaseline(final String target);

  /**
   * Checks whether the provided upgrade step has been applied
   */
  boolean isMigrationApplied(Class<? extends DatabaseMigrationStep> step);
}
