
package org.globalart.astrum.repository.content.upgrades;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Component;

@Component
public class ConanCleanupMigrationStep_2_18
    implements DatabaseMigrationStep
{
  private static final Logger LOG = LoggerFactory.getLogger(ConanCleanupMigrationStep_2_18.class);

  static final String TABLE_NAME = "conan_component";

  static final String OLD_INDEX_NAME = "idx_conan_component_coordinate_revisions";

  static final String OLD_CONSTRAINT_NAME = "uk_conan_component_coordinate_revisions";

  static final String OLD_REVISION_COLUMN_NAME = "revision";

  static final String OLD_REVISION_TIME_COLUMN_NAME = "revision_time";

  static final String NEW_CONSTRAINT_NAME = "uk_conan_component_coordinate";

  private static final String DROP_INDEX = String.format("DROP INDEX IF EXISTS %s;", OLD_INDEX_NAME);

  private static final String DROP_CONSTRAINT =
      String.format("ALTER TABLE IF EXISTS %s DROP CONSTRAINT IF EXISTS %s;", TABLE_NAME, OLD_CONSTRAINT_NAME);

  private static final String DROP_COLUMN_REVISION =
      String.format("ALTER TABLE IF EXISTS %s DROP COLUMN IF EXISTS %s;", TABLE_NAME, OLD_REVISION_COLUMN_NAME);

  private static final String DROP_COLUMN_REVISION_TIME =
      String.format("ALTER TABLE IF EXISTS %s DROP COLUMN IF EXISTS %s;", TABLE_NAME, OLD_REVISION_TIME_COLUMN_NAME);

  private static final String ADD_CONSTRAINT =
      String.format("ALTER TABLE IF EXISTS %s ADD CONSTRAINT %s UNIQUE (repository_id, namespace, name, version)",
          TABLE_NAME, NEW_CONSTRAINT_NAME);

  @Override
  public Optional<String> version() {
    return Optional.of("2.18");
  }

  /**
   * We had an index, as well as 2 columns (revision and revision_time) in the conan_component table, and a constraint
   * that utilized one of those columns. These were all from an early iteration of conan revision support, and are no
   * longer needed. This migration step removes the index, the columns, and the constraint, replacing it with a new
   * constraint that does not use the revision column.
   */
  @Override
  public void migrate(final Connection connection) throws Exception {
    LOG.info("Removing outdated indexes and columns from {}.", TABLE_NAME);

    this.runStatement(connection, DROP_INDEX);
    LOG.debug("Drop index {} on {} (IF EXISTS)", OLD_INDEX_NAME, TABLE_NAME);

    boolean constraintExists = constraintExists(connection, TABLE_NAME, OLD_CONSTRAINT_NAME);
    if (constraintExists) {
      this.runStatement(connection, DROP_CONSTRAINT);
      LOG.debug("Drop constraint {} from table {} (IF EXISTS)", OLD_CONSTRAINT_NAME, TABLE_NAME);
      this.runStatement(connection, ADD_CONSTRAINT);
      LOG.debug("Added constraint {} to table {}", NEW_CONSTRAINT_NAME, TABLE_NAME);
    }

    this.runStatement(connection, DROP_COLUMN_REVISION);
    LOG.debug("Drop column {} from table {} (IF EXISTS)", OLD_REVISION_COLUMN_NAME, TABLE_NAME);

    this.runStatement(connection, DROP_COLUMN_REVISION_TIME);
    LOG.debug("Drop column {} from table {} (IF EXISTS)", OLD_REVISION_TIME_COLUMN_NAME, TABLE_NAME);

    LOG.info("Successfully completed correcting the state of the {} table.", TABLE_NAME);
  }
}
