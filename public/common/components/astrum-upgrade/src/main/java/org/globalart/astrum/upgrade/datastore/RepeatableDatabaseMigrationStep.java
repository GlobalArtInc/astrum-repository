
package org.globalart.astrum.upgrade.datastore;

import java.util.Optional;

/**
 * Support class for repeatable migrations
 */
public interface RepeatableDatabaseMigrationStep
    extends DatabaseMigrationStep
{
  @Override
  default Optional<String> version() {
    // Do not override me.
    return Optional.empty();
  }

  /**
   * Migrations will run when the checksum changes.
   *
   * Flyway considers the checksum prior to the first run as {@code null}
   */
  @Override
  Integer getChecksum();
}
