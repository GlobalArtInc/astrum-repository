
package org.globalart.astrum.upgrade.datastore.internal;

import java.util.function.Predicate;
import java.util.regex.Pattern;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.globalart.astrum.upgrade.datastore.DefinedUpgradeRound;

import org.flywaydb.core.api.MigrationVersion;
import org.flywaydb.core.api.migration.Context;
import org.flywaydb.core.api.migration.JavaMigration;

/**
 * Wrapper for {@link DatabaseMigrationStep} for Flyway
 *
 * @since 3.29
 */
public class AstrumJavaMigration
    implements JavaMigration
{
  private final DatabaseMigrationStep dbMigrationStep;

  private final Integer round;

  public AstrumJavaMigration(final DatabaseMigrationStep dbMigrationStep) {
    this(dbMigrationStep, null);
  }

  public AstrumJavaMigration(final DatabaseMigrationStep dbMigrationStep, final Integer round) {
    this.dbMigrationStep = dbMigrationStep;

    if (dbMigrationStep instanceof DefinedUpgradeRound) {
      if (round != null) {
        throw new IllegalStateException("Incompatible API, DefinedUpgradeRound cannot be mixd with DependsOn");
      }
      this.round = ((DefinedUpgradeRound) dbMigrationStep).getUpgradeRound();
    }
    else {
      this.round = round;
    }
  }

  @Override
  public boolean canExecuteInTransaction() {
    return dbMigrationStep.canExecuteInTransaction();
  }

  @Override
  public Integer getChecksum() {
    return dbMigrationStep.getChecksum();
  }

  @Override
  public String getDescription() {
    // We need to remove the Guice suffixes to achieve a stable name,
    // e.g. S3BlobStoreMetricsMigrationStep$$EnhancerByGuice$$2c7e99a6
    String className = dbMigrationStep.getClass().getSimpleName().split("\\$\\$")[0];

    if (round != null) {
      return String.format("Z_%03d_%s", round, className);
    }
    return className;
  }

  @Override
  public MigrationVersion getVersion() {
    return dbMigrationStep.version()
        .map(MigrationVersion::fromVersion)
        .orElse(null);
  }

  @Override
  public void migrate(final Context context) throws Exception {
    dbMigrationStep.migrate(context.getConnection());
  }

  /**
   * Provides a {@link Predicate} which will match the against flyway migration descriptions and also accounts for
   * a possible round prefix.
   */
  public static Predicate<String> nameMatcher(final Class<? extends DatabaseMigrationStep> step) {
    return Pattern.compile("^(Z_\\d{3}_)?" + step.getSimpleName() + "$").asPredicate();
  }
}
