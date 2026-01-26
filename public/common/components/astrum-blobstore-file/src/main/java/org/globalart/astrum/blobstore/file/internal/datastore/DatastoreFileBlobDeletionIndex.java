
package org.globalart.astrum.blobstore.file.internal.datastore;

import java.io.File;
import java.io.IOException;
import java.time.Duration;
import java.util.Objects;
import java.util.Set;
import java.util.stream.Collectors;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.blobstore.api.BlobId;
import org.globalart.astrum.blobstore.api.softdeleted.SoftDeletedBlobsStore;
import org.globalart.astrum.blobstore.file.FileBlobDeletionIndex;
import org.globalart.astrum.blobstore.file.FileBlobStore;
import org.globalart.astrum.blobstore.internal.SoftDeletedBlobIndexImpl;
import org.globalart.astrum.common.property.PropertiesFile;
import org.globalart.astrum.common.scheduling.PeriodicJobService;
import org.globalart.astrum.logging.task.ProgressLogIntervalHelper;

import com.squareup.tape.QueueFile;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;
import static java.nio.charset.StandardCharsets.UTF_8;
import static org.globalart.astrum.blobstore.file.FileBlobStore.REBUILD_DELETED_BLOB_INDEX_KEY;

@Component
@Qualifier(FileBlobStore.TYPE)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class DatastoreFileBlobDeletionIndex
    extends SoftDeletedBlobIndexImpl
    implements FileBlobDeletionIndex
{
  private static final int INTERVAL_IN_SECONDS = 60;

  private final PeriodicJobService periodicJobService;

  private final Duration migrationDelay;

  private FileBlobStore blobStore;

  @Autowired
  public DatastoreFileBlobDeletionIndex(
      final SoftDeletedBlobsStore softDeletedBlobsStore,
      final PeriodicJobService periodicJobService,
      @Value("${astrum.file.deletion.migrate.delay:60s}") final Duration migrationDelay)
  {
    super(softDeletedBlobsStore);
    this.periodicJobService = checkNotNull(periodicJobService);
    this.migrationDelay = checkNotNull(migrationDelay);
    checkArgument(!migrationDelay.isNegative(), "Non-negative astrum.file.deletion.migrate.delay required");
  }

  @Override
  public final void initIndex(final PropertiesFile metadata, final FileBlobStore blobStore) throws IOException {
    this.blobStore = blobStore;
    init(blobStore);
    scheduleMigrateIndex(metadata);
  }

  @Override
  public void stopIndex() throws IOException {
    // no special procedure is required
  }

  private void scheduleMigrateIndex(final PropertiesFile metadata) {
    periodicJobService.runOnce(() -> {
      try {
        migrateDeletionIndexFromFiles(metadata);
      }
      catch (IOException e) {
        log.error("Failed to migrate soft deleted blobs to the database", e);
      }
    }, (int) migrationDelay.getSeconds());
  }

  private void migrateDeletionIndexFromFiles(final PropertiesFile metadata) throws IOException {
    blobStore.getDeletionIndexFiles()
        .forEach(deletionIndexFile -> {
          try {
            migrateDeletionIndexFromFile(metadata, deletionIndexFile);
          }
          catch (IOException e) {
            log.error("An error occurred while attempting to migrate the deletions index {} for {}", deletionIndexFile,
                blobStoreName);
          }
        });
  }

  private void migrateDeletionIndexFromFile(
      final PropertiesFile metadata,
      final @Nullable File oldDeletionIndexFile) throws IOException
  {
    QueueFile oldDeletionIndex;
    log.debug("Starting migration in {} for {}", blobStoreName, oldDeletionIndexFile);

    try {
      oldDeletionIndex = new QueueFile(oldDeletionIndexFile);
    }
    catch (IOException e) {
      log.error(
          "Unable to load deletions index file {}, run the compact blobstore task to rebuild", oldDeletionIndexFile, e);
      oldDeletionIndex = null;
      metadata.setProperty(REBUILD_DELETED_BLOB_INDEX_KEY, "true");
      metadata.store();
    }

    if (Objects.nonNull(oldDeletionIndex) && !oldDeletionIndex.isEmpty()) {
      log.info("Processing blobstore {}, discovered file-based deletion index. Migrating to DB-based",
          blobStore.getBlobStoreConfiguration().getName());
      Set<BlobId> persistedRecords = softDeletedBlobsStore.readAllBlobIds(blobStoreName)
          .collect(Collectors.toSet());

      try (ProgressLogIntervalHelper progressLogger = new ProgressLogIntervalHelper(log, INTERVAL_IN_SECONDS)) {
        for (int counter = 0, numBlobs = oldDeletionIndex.size(); counter < numBlobs; counter++) {
          byte[] bytes = oldDeletionIndex.peek();
          if (bytes == null) {
            log.debug("Queue indicated no more results {} of {}", counter, numBlobs);
            break;
          }
          BlobId blobId = new BlobId(new String(bytes, UTF_8));
          if (!persistedRecords.contains(blobId)) {
            softDeletedBlobsStore.createRecord(blobId, blobStore.getBlobStoreConfiguration().getName());
            persistedRecords.add(blobId);
          }
          else {
            log.debug("Old deletion index contain duplicate entry with blobId - {} for blobstore - {}, " +
                "duplicate record will be skipped", blobId, blobStoreName);
          }

          oldDeletionIndex.remove();
          progressLogger.info("Elapsed time: {}, processed: {}/{}", progressLogger.getElapsed(),
              counter + 1, numBlobs);
        }
      }
    }

    if (oldDeletionIndexFile.exists() && !oldDeletionIndexFile.delete()) {
      log.error("Unable to delete 'deletion index' file, path = {}", oldDeletionIndexFile.getAbsolutePath());
    }
  }
}
