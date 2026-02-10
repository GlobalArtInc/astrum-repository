
package org.globalart.astrum.repository.content.upgrades;

import java.sql.Connection;
import java.sql.Statement;
import java.util.Objects;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Migration step to create an index for idx_soft_deleted_blobs_by_source_blob_store_name_record_id
 */
@Component
public class SoftDeletedBlobsByBlobStoreIndexMigrationStep_2_7
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final String ADD_INDEX_STATEMENT =
      "CREATE INDEX IF NOT EXISTS idx_soft_deleted_blobs_by_source_blob_store_name_record_id"
          + " ON soft_deleted_blobs (source_blob_store_name, record_id);";

  @Override
  public Integer getChecksum() {
    return Objects.hash(ADD_INDEX_STATEMENT);
  }

  @Override
  public Optional<String> version() {
    return Optional.of("2.7");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    long startTime = System.currentTimeMillis();
    log.info("Creating index idx_soft_deleted_blobs_by_source_blob_store_name_record_id");

    try (Statement statement = connection.createStatement()) {
      statement.execute(ADD_INDEX_STATEMENT);
    }

    log.info("Index idx_soft_deleted_blobs_by_source_blob_store_name_record_id created successfully in {} seconds.",
        (System.currentTimeMillis() - startTime) * 0.001d);
  }
}
