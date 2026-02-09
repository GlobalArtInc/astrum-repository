
package org.globalart.astrum.repository.content.tasks;

import java.sql.Connection;
import java.sql.SQLException;
import java.util.List;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationUtility;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;
import org.springframework.stereotype.Component;

/**
 * Service to remove the older component indexes on repository_id, namespace, name. And add new indexes on
 * repository_id, namespace, name AND version.
 */
@Component
public class CreateComponentIndexService
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final String TABLE_NAME = "{format}_component";

  private final String OLD_INDEX_NAME = "idx_{format}_component_set";

  private final String NEW_INDEX_NAME = "idx_{format}_component_coordinates";

  private final String COLUMNS = "repository_id, namespace, name, version";

  private final String FORMAT_KEY = "{format}";

  private final DataSessionSupplier dataSessionSupplier;

  private final DatabaseMigrationUtility databaseMigrationUtility;

  private final List<Format> formats;

  @Autowired
  public CreateComponentIndexService(
      final DataSessionSupplier dataSessionSupplier,
      final DatabaseMigrationUtility databaseMigrationUtility,
      final List<Format> formats)
  {
    this.dataSessionSupplier = checkNotNull(dataSessionSupplier);
    this.databaseMigrationUtility = checkNotNull(databaseMigrationUtility);
    this.formats = checkNotNull(formats);
  }

  public void recreateComponentIndexes() throws SQLException {
    int totalCount = formats.size();

    try (Connection connection = dataSessionSupplier.openConnection(DEFAULT_DATASTORE_NAME)) {
      for (int i = 0; i < totalCount; i++) {
        Format format = formats.get(i);
        log.info(
            "Recreating component coordinate index for format: {}.  {}% done",
            format.getValue(),
            Math.round(((float) i / totalCount) * 100));
        processFormat(connection, format);
      }
      log.info("Recreating component coordinate index for all formats.  100% done");
    }
  }

  private void processFormat(final Connection connection, final Format format) {
    if (createNewIndex(connection, format)) {
      deleteExistingIndex(connection, format);
    }
    else {
      log.error(
          "Will not remove existing component coordinate index as creation of the new index failed for format: {}",
          format.getValue());
    }
  }

  private boolean deleteExistingIndex(final Connection connection, final Format format) {
    String indexName = OLD_INDEX_NAME.replace(FORMAT_KEY, format.getValue());
    try {
      databaseMigrationUtility.dropIndex(connection, indexName);
      return true;
    }
    catch (SQLException e) {
      log.error("Failed to delete existing index {}", indexName, e);
    }
    return false;
  }

  private boolean createNewIndex(final Connection connection, final Format format) {
    String indexName = NEW_INDEX_NAME.replace(FORMAT_KEY, format.getValue());
    String tableName = TABLE_NAME.replace(FORMAT_KEY, format.getValue());
    try {
      databaseMigrationUtility.createIndex(connection, indexName, tableName, COLUMNS);
      return true;
    }
    catch (SQLException e) {
      log.error("Failed to create new index {}", indexName, e);
    }
    return false;
  }
}
