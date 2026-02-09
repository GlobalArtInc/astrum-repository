
package org.globalart.astrum.repository.content.tasks;

import java.sql.Connection;
import java.sql.PreparedStatement;
import java.sql.SQLException;
import java.util.List;

import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.scheduling.TaskSupport;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * Creates composite index on {format}_asset_blob tables for blob_created DESC and asset_blob_id ASC.
 * This task only creates the index if it doesn't already exist.
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class CreateAssetBlobIndexTask
    extends TaskSupport
{
  private static final String CREATE_INDEX_PG = "CREATE INDEX CONCURRENTLY ";

  private static final String CREATE_INDEX_H2 = "CREATE INDEX ";

  private static final String INDEX_NAME_PATTERN = "idx_%s_asset_blob_blob_created_asset_id";

  private static final String INDEX_DEF_PATTERN = " ON %s_asset_blob (blob_created DESC, asset_blob_id ASC);";

  private final List<Format> formats;

  private final DataSessionSupplier dataSessionSupplier;

  @Autowired
  public CreateAssetBlobIndexTask(
      final List<Format> formats,
      final DataSessionSupplier dataSessionSupplier)
  {
    this.formats = checkNotNull(formats);
    this.dataSessionSupplier = checkNotNull(dataSessionSupplier);
  }

  @Override
  public String getMessage() {
    return "Creating asset blob indexes for blob_created and asset_blob_id";
  }

  @Override
  protected Object execute() throws Exception {
    int indexesCreated = 0;
    List<Exception> collectedExceptions = new java.util.ArrayList<>();

    try (Connection connection = dataSessionSupplier.openConnection("astrum")) {
      boolean isH2 = isH2(connection);
      String createIndexClause = isH2 ? CREATE_INDEX_H2 : CREATE_INDEX_PG;

      for (Format format : formats) {
        String tableName = format.getValue() + "_asset_blob";
        String indexName = INDEX_NAME_PATTERN.formatted(format.getValue());

        try {
          if (tableExists(connection, tableName) && !indexExists(connection, tableName, indexName)) {
            String indexDefinition = INDEX_NAME_PATTERN.formatted(format.getValue()) +
                INDEX_DEF_PATTERN.formatted(format.getValue());

            try {
              executeStatement(connection, createIndexClause + indexDefinition);
              indexesCreated++;
              log.info("Created index {} on table {}", indexName, tableName);
            }
            catch (SQLException e) {
              log.error("Failed to create index '{}' on table '{}'. Attempting to clean up invalid index.",
                  indexName, tableName, e);
              // If CREATE INDEX CONCURRENTLY fails, it may leave behind an invalid index that needs cleanup
              dropInvalidIndex(connection, indexName, isH2);
              collectedExceptions.add(e);
            }
          }
          else {
            log.debug("Index {} already exists on table {} or table doesn't exist", indexName, tableName);
          }
        }
        catch (SQLException e) {
          log.error("Failed to create index '{}' on table '{}'", indexName, tableName, e);
          collectedExceptions.add(e);
        }
      }
    }

    log.info("Created {} asset blob indexes", indexesCreated);

    // If there were any failures, throw an exception with all causes attached, this will allow task to
    // execute again when instance restarts
    if (!collectedExceptions.isEmpty()) {
      Exception aggregatedException = new Exception(
          "Failed to create %d index(es) out of %d format(s). Task will try again later.".formatted(
              collectedExceptions.size(), formats.size()));
      for (Exception e : collectedExceptions) {
        aggregatedException.addSuppressed(e);
      }
      throw aggregatedException;
    }

    return indexesCreated;
  }

  private void executeStatement(final Connection connection, final String sqlStatement) throws SQLException {
    try (PreparedStatement statement = connection.prepareStatement(sqlStatement)) {
      statement.executeUpdate();
    }
  }

  private void dropInvalidIndex(final Connection connection, final String indexName, final boolean isH2) {
    try {
      if (!isH2) {
        // PostgreSQL: Check if index exists and is invalid, then drop it
        String checkInvalidSql = "SELECT 1 FROM pg_indexes WHERE indexname = ?";
        try (PreparedStatement stmt = connection.prepareStatement(checkInvalidSql)) {
          stmt.setString(1, indexName.toLowerCase(java.util.Locale.ENGLISH));
          try (java.sql.ResultSet rs = stmt.executeQuery()) {
            if (rs.next()) {
              String dropSql = "DROP INDEX CONCURRENTLY IF EXISTS " + indexName;
              executeStatement(connection, dropSql);
              log.info("Dropped invalid index: {}", indexName);
            }
          }
        }
      }
      else {
        // H2: Simply try to drop the index if it exists
        String dropSql = "DROP INDEX IF EXISTS " + indexName;
        executeStatement(connection, dropSql);
        log.info("Dropped invalid index: {}", indexName);
      }
    }
    catch (SQLException e) {
      log.warn("Failed to drop invalid index '{}', it may need manual cleanup", indexName, e);
      // Don't throw - this is best-effort cleanup
    }
  }

  private boolean isH2(final Connection conn) throws SQLException {
    return "H2".equals(conn.getMetaData().getDatabaseProductName());
  }

  private boolean tableExists(final Connection conn, final String tableName) throws SQLException {
    if (isH2(conn)) {
      try (PreparedStatement statement =
          conn.prepareStatement("SELECT * FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_NAME = ?")) {
        statement.setString(1, tableName.toUpperCase());
        try (java.sql.ResultSet results = statement.executeQuery()) {
          return results.next();
        }
      }
    }
    else {
      try (PreparedStatement statement = conn.prepareStatement("SELECT to_regclass(?);")) {
        statement.setString(1, tableName);
        try (java.sql.ResultSet results = statement.executeQuery()) {
          if (!results.next()) {
            return false;
          }
          Object oid = results.getObject(1);
          return oid != null;
        }
      }
    }
  }

  private boolean indexExists(
      final Connection conn,
      final String tableName,
      final String indexName) throws SQLException
  {
    if (!isH2(conn)) {
      String sql = "SELECT * FROM pg_indexes WHERE UPPER(tablename) = ? AND UPPER(indexname) = ?";

      try (PreparedStatement statement = conn.prepareStatement(sql)) {
        statement.setString(1, tableName.toUpperCase(java.util.Locale.ENGLISH));
        statement.setString(2, indexName.toUpperCase(java.util.Locale.ENGLISH));
        try (java.sql.ResultSet results = statement.executeQuery()) {
          return results.next();
        }
      }
    }
    else {
      String sql =
          "SELECT * FROM information_schema.indexes WHERE REGEXP_LIKE(UPPER(index_name), ?) AND UPPER(TABLE_NAME) = ?";
      try (PreparedStatement statement = conn.prepareStatement(sql)) {
        statement.setString(1, (indexName.toUpperCase(java.util.Locale.ENGLISH) + "[_[0-9]*]?"));
        statement.setString(2, tableName.toUpperCase(java.util.Locale.ENGLISH));
        try (java.sql.ResultSet results = statement.executeQuery()) {
          return results.next();
        }
      }
    }
  }
}
