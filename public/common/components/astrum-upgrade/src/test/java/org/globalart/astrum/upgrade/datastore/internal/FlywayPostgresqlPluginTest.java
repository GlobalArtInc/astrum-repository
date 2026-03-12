
package org.globalart.astrum.upgrade.datastore.internal;

import javax.sql.DataSource;

import org.globalart.astrum.testdb.DataSessionConfiguration;
import org.globalart.astrum.testdb.DatabaseTest;
import org.globalart.astrum.testdb.TestDataSessionSupplier;

import org.flywaydb.core.Flyway;
import org.junit.jupiter.api.extension.ExtendWith;
import org.globalart.astrum.testdb.DatabaseExtension;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;

/**
 * Verifies that {@code flyway-database-postgresql} ServiceLoader plugin is on the classpath
 * and activates correctly when Flyway connects to a PostgreSQL datasource.
 *
 * Flyway 10+ split PostgreSQL support into a separate artifact. Without it on the classpath,
 * {@code flyway.info()} throws "No database found to handle jdbc:postgresql://..." at runtime.
 */
@ExtendWith(DatabaseExtension.class)
class FlywayPostgresqlPluginTest
{
  @DataSessionConfiguration(daos = {}, postgresql = true)
  TestDataSessionSupplier supplier;

  @DatabaseTest(postgresql = true, h2 = false)
  void postgresqlPluginLoads() {
    DataSource dataSource = supplier.getDataSource()
        .orElseThrow(() -> new IllegalStateException("No DataSource available"));

    Flyway flyway = Flyway.configure()
        .dataSource(dataSource)
        .locations(new String[0]) // no script scanning — same as UpgradeManagerImpl
        .cleanDisabled(true)
        .load();

    assertDoesNotThrow(flyway::info,
        "flyway-database-postgresql plugin must be on the classpath and load via ServiceLoader " +
            "— if this fails with 'No database found to handle jdbc:postgresql://' the runtime artifact is missing");
  }
}
