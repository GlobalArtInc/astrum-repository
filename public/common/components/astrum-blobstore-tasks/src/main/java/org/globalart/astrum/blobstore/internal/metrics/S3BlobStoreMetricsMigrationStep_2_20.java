
package org.globalart.astrum.blobstore.internal.metrics;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.stereotype.Component;

/**
 * Dummy no-op migration step to maintain database migration history consistency.
 *
 * This step was temporarily created as a versioned migration step and executed in some environments
 * before being reverted. To prevent "Missing migrations" errors in those environments, this dummy
 * step is kept in the codebase but performs no operations.
 *
 * Original functionality was moved to the repeatable S3BlobStoreMetricsMigrationStep.
 */
@Component
public class S3BlobStoreMetricsMigrationStep_2_20
    implements DatabaseMigrationStep
{
  @Override
  public Optional<String> version() {
    return Optional.of("2.20");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    // No-op: This is a placeholder step to maintain migration history consistency
  }
}
