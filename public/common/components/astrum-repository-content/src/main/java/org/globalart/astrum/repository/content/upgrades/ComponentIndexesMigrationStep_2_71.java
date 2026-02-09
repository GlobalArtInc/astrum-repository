
package org.globalart.astrum.repository.content.upgrades;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import com.google.common.annotations.VisibleForTesting;
import java.sql.Connection;
import java.sql.SQLException;
import java.util.List;
import java.util.Optional;
import org.springframework.stereotype.Component;

/**
 * Adds an index on {format}_component tables for the "repository_id, name, component_id" combination.
 *
 * This index will get used on queries using the following form:
 * 
 * <pre>
 *   SELECT * FROM {format}_component
 *   WHERE repository_id = $1
 *     AND name = $2
 *   ORDER BY component_id
 *   LIMIT $3
 * </pre>
 *
 * See ComponentDAO#browseComponentsBySet. That query as written will include the namespace column, but for
 * a number of formats the namespace will commonly be empty. We need an index on repository_id, name
 * and component_id for efficient ordered results.
 */
@Component
public class ComponentIndexesMigrationStep_2_71
    implements DatabaseMigrationStep
{
  private final List<Format> formats;

  @Autowired
  public ComponentIndexesMigrationStep_2_71(final List<Format> formats) {
    this.formats = formats;
  }

  @Override
  public Optional<String> version() {
    return Optional.of("2.71");
  }

  @Override
  public boolean canExecuteInTransaction() {
    return false; // this is required to use CONCURRENTLY in the CREATE INDEX for postgresql
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    for (Format format : formats) {
      String tableName = format.getValue() + "_component";
      if (tableExists(connection, tableName)) {
        createIndexes(connection, format.getValue(), tableName);
      }
    }
  }

  @VisibleForTesting
  void createIndexes(final Connection connection, final String format, final String tableName) throws SQLException {
    // CREATE INDEX CONCURRENTLY idx_format_component_repository_component_id
    // ON format_component (repository_id, name, component_id);
    String indexName = "idx_" + format + "_component_repository_component_id";
    String columns = "repository_id, name, component_id";
    if (indexExists(connection, tableName, indexName)) {
      return;
    }

    if (isPostgresql(connection)) {
      runStatement(connection,
          String.format("CREATE INDEX CONCURRENTLY IF NOT EXISTS %s ON %s (%s)", indexName, tableName, columns));
    }
    else if (isH2(connection)) {
      runStatement(connection,
          String.format("CREATE INDEX IF NOT EXISTS %s ON %s (%s)", indexName, tableName, columns));
    }
  }
}
