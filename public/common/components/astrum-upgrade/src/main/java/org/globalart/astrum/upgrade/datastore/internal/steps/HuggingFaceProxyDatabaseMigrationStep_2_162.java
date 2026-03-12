
package org.globalart.astrum.upgrade.datastore.internal.steps;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.stereotype.Component;

/**
 * Availability marker for HuggingFace XET per-repository config attributes
 * ({@code xetCasUrl}, {@code xetTransferHostAllowList}) added in ASTRUM-48685.
 *
 * <p>
 * These attributes live inside the shared {@code repository.attributes} JSON column, so no
 * schema change is required. The step exists purely to advance the Flyway version so that
 * {@code @AvailabilityVersion(from = "2.162")} on the HuggingFace proxy recipe activates once
 * the migration has run.
 *
 * <p>
 * <b>Version history:</b> originally 2.161. Renumbered to 2.162 when ASTRUM-54219
 * ({@code SearchComponentsVersionBrowseIndexMigrationStep_2_161}) merged to main and took
 * the 2.161 slot. {@code MigrationVersionUniquenessTest} enforces that no two migrations
 * share a version.
 */
@Component
public class HuggingFaceProxyDatabaseMigrationStep_2_162
    implements DatabaseMigrationStep
{
  @Override
  public Optional<String> version() {
    return Optional.of("2.162");
  }

  @Override
  public void migrate(final Connection connection) {
    // No-op, this makes the HuggingFace XET per-repository config attributes available
  }
}
