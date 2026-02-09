
package org.globalart.astrum.repository.content.upgrades;

import java.sql.Connection;
import java.sql.SQLException;
import java.util.List;
import java.util.Optional;
import java.util.Set;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

/**
 * Adds composite index for retain N cleanup queries on all retain-enabled format component tables.
 *
 * Index covers the full retain query access pattern: repository_id, namespace, name, normalized_version DESC.
 * This allows efficient ORDER BY + OFFSET without sort-on-disk.
 *
 * Related to ASTRUM-52740 - Cleanup Policy Enhancements (Retain N for all formats).
 */
@Component
public class RetainNCompositeIndexMigrationStep_2_144
    implements DatabaseMigrationStep
{
  private static final Set<String> RETAIN_FORMATS = Set.of(
      "maven2", "docker", "npm", "pypi", "go", "helm", "nuget",
      "yum", "rubygems", "terraform", "swift", "apt", "pub");

  private final List<Format> formats;

  @Autowired
  public RetainNCompositeIndexMigrationStep_2_144(final List<Format> formats) {
    this.formats = formats;
  }

  @Override
  public Optional<String> version() {
    return Optional.of("2.144");
  }

  @Override
  public boolean canExecuteInTransaction() {
    return false; // this is required to use CONCURRENTLY in the CREATE INDEX for postgresql
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    for (Format format : formats) {
      String formatName = format.getValue();
      if (!RETAIN_FORMATS.contains(formatName)) {
        continue;
      }
      String tableName = formatName + "_component";
      if (tableExists(connection, tableName)) {
        createRetainIndex(connection, formatName, tableName);
      }
    }
  }

  private void createRetainIndex(
      final Connection connection,
      final String format,
      final String tableName) throws SQLException
  {
    String indexName = "idx_" + format + "_component_retain";
    if (indexExists(connection, tableName, indexName)) {
      return;
    }
    if (isPostgresql(connection)) {
      runStatement(connection,
          String.format(
              "CREATE INDEX CONCURRENTLY IF NOT EXISTS %s ON %s (repository_id, namespace, name, normalized_version DESC)",
              indexName, tableName));
    }
    else if (isH2(connection)) {
      runStatement(connection,
          String.format("CREATE INDEX IF NOT EXISTS %s ON %s (repository_id, namespace, name, normalized_version DESC)",
              indexName, tableName));
    }
  }
}
