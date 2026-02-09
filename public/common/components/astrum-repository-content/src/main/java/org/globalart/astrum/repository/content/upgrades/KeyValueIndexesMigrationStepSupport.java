

package org.globalart.astrum.repository.content.upgrades;

import java.sql.Connection;
import java.sql.SQLException;
import java.util.List;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Adds indexes to {format}_key_value tables for existing databases.
 */
public abstract class KeyValueIndexesMigrationStepSupport
    implements DatabaseMigrationStep
{
  private final List<String> formats;

  /**
   * @param formats a list of formats to create indexes for if required
   */
  protected KeyValueIndexesMigrationStepSupport(final List<String> formats) {
    this.formats = checkNotNull(formats);
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    for (String format : formats) {
      String tableName = format + "_key_value";
      if (tableExists(connection, tableName)) {
        createIndexes(connection, format, tableName);
      }
    }
  }

  private void createIndexes(
      final Connection connection,
      final String format,
      final String tableName) throws SQLException
  {
    // Note: 'key' is a reserved keyword in H2, so we need to escape it with backticks
    String keyColumn = isH2(connection) ? "`key`" : "key";

    String uniqueIndexName = "uk_" + format + "_repository_category_key";
    String regularIndexName = "idx_" + format + "_key_value_repository_key";

    createUniqueIndexIfNotExists(connection, uniqueIndexName, tableName, "repository_id, category, " + keyColumn);

    createIndexIfNotExists(connection, regularIndexName, tableName, "repository_id, " + keyColumn);
  }

  private void createUniqueIndexIfNotExists(
      final Connection connection,
      final String indexName,
      final String tableName,
      final String columns) throws SQLException
  {
    if (indexExists(connection, tableName, indexName)) {
      return;
    }

    if (isPostgresql(connection)) {
      runStatement(connection, String.format("""
          DO $$
          BEGIN
            BEGIN
              CREATE UNIQUE INDEX %s ON %s (%s);
            EXCEPTION
              WHEN duplicate_table THEN NULL;
            END;
          END $$;
          """, indexName, tableName, columns));
    }
    else if (isH2(connection)) {
      runStatement(connection,
          String.format("CREATE UNIQUE INDEX IF NOT EXISTS %s ON %s (%s)", indexName, tableName, columns));
    }
  }

  private void createIndexIfNotExists(
      final Connection connection,
      final String indexName,
      final String tableName,
      final String columns) throws SQLException
  {
    if (indexExists(connection, tableName, indexName)) {
      return;
    }

    if (isPostgresql(connection)) {
      runStatement(connection, String.format("""
          DO $$
          BEGIN
            BEGIN
              CREATE INDEX %s ON %s (%s);
            EXCEPTION
              WHEN duplicate_table THEN NULL;
            END;
          END $$;
          """, indexName, tableName, columns));
    }
    else if (isH2(connection)) {
      runStatement(connection,
          String.format("CREATE INDEX IF NOT EXISTS %s ON %s (%s)", indexName, tableName, columns));
    }
  }
}
