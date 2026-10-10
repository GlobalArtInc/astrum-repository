package org.globalart.astrum.internal.security.model.upgrade;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.stereotype.Component;

/**
 * Keeps schema version 2.165, which existing databases already record; there is nothing left to migrate.
 */
@Component
public class SecurityIdPrefixMigrationStep_2_165
    implements DatabaseMigrationStep
{
  @Override
  public Optional<String> version() {
    return Optional.of("2.165");
  }

  @Override
  public void migrate(final Connection connection) {
    // nothing to migrate
  }
}
