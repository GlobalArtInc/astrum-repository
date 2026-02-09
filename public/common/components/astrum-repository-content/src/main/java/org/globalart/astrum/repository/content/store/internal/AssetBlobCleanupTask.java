
package org.globalart.astrum.repository.content.store.internal;

import com.google.common.annotations.VisibleForTesting;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.concurrent.ConcurrentLinkedQueue;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.ExecutorService;
import java.util.stream.Collectors;
import org.springframework.beans.factory.annotation.Autowired;
import javax.annotation.Nullable;

import org.globalart.astrum.blobstore.api.Blob;
import org.globalart.astrum.blobstore.api.BlobRef;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.scheduling.RecoveryModeService;
import org.globalart.astrum.logging.task.TaskLogging;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.content.store.AssetBlobStore;
import org.globalart.astrum.repository.content.store.BlobRefTypeHandler;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.globalart.astrum.scheduling.CancelableByRecoveryMode;
import org.globalart.astrum.scheduling.TaskSupport;
import org.globalart.astrum.thread.AstrumThreadFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.concurrent.Executors.newFixedThreadPool;
import static org.globalart.astrum.common.property.SystemPropertiesHelper.getBoolean;
import static org.globalart.astrum.common.property.SystemPropertiesHelper.getInteger;
import static org.globalart.astrum.common.property.SystemPropertiesHelper.getString;
import static org.globalart.astrum.logging.task.TaskLogType.ASTRUM_LOG_ONLY;
import static org.globalart.astrum.repository.content.store.internal.AssetBlobCleanupTaskDescriptor.CONTENT_STORE_FIELD_ID;
import static org.globalart.astrum.repository.content.store.internal.AssetBlobCleanupTaskDescriptor.FORMAT_FIELD_ID;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * Background task that cleans up unused {@link AssetBlob}s.
 *
 * @since 3.24
 */
@Component
@TaskLogging(ASTRUM_LOG_ONLY)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class AssetBlobCleanupTask
    extends TaskSupport
    implements CancelableByRecoveryMode
{
  static final String PROPERTY_PREFIX = "astrum.assetBlobCleanupTask.";

  static final String CRON_SCHEDULE = getString(PROPERTY_PREFIX + "cronSchedule", "0 */30 * * * ?");

  static final int BATCH_SIZE = getInteger(PROPERTY_PREFIX + "batchSize", 1000);

  static final boolean HARD_DELETE = getBoolean(PROPERTY_PREFIX + "hardDelete", false);

  static final int BLOB_CREATED_DELAY_MINUTE =
      getInteger(PROPERTY_PREFIX + "blobCreatedDelayMinute", 60);

  static final int BATCH_DELETE_POOL_SIZE = getInteger(
      PROPERTY_PREFIX + "batchDeleteThreadPoolSize", 8);

  private final Map<String, FormatStoreManager> formatStoreManagers;

  private final BlobStoreManager blobStoreManager;

  private final RecoveryModeService recoveryModeService;

  private Boolean batchDeleteEnabled = true;

  private ExecutorService batchDeleteExecutorService;

  @Autowired
  public AssetBlobCleanupTask(
      final List<FormatStoreManager> formatStoreManagersList,
      final BlobStoreManager blobStoreManager,
      @Nullable final RecoveryModeService recoveryModeService)
  {
    this.formatStoreManagers = QualifierUtil.buildQualifierBeanMap(checkNotNull(formatStoreManagersList));
    this.blobStoreManager = checkNotNull(blobStoreManager);
    this.recoveryModeService = recoveryModeService;
  }

  protected void initBatchDeleteIfEnabled(final String format) {
    String batchDeleteIgnoreFormats =
        getString(PROPERTY_PREFIX + "batchDeleteIgnoreForFormat", null);

    if (batchDeleteIgnoreFormats != null && format != null
        && batchDeleteIgnoreFormats.contains(format)) {
      batchDeleteEnabled = false;
    }
    else {
      batchDeleteExecutorService = newFixedThreadPool(
          BATCH_DELETE_POOL_SIZE,
          new AstrumThreadFactory("blobstore", "async-ops"));
    }
  }

  @Override
  protected Void execute() throws Exception {
    if (recoveryModeService != null) {
      recoveryModeService.ensureNotInRecoveryMode(getName());
    }

    String format = getConfiguration().getString(FORMAT_FIELD_ID);
    String contentStore = getConfiguration().getString(CONTENT_STORE_FIELD_ID);
    initBatchDeleteIfEnabled(format);

    FormatStoreManager formatStoreManager = formatStoreManagers.get(format);
    if (formatStoreManager != null) {
      log.debug("Checking for unused {} blobs from {}", format, contentStore);
      AssetBlobStore<?> assetBlobStore = formatStoreManager.assetBlobStore(contentStore);
      int deleteCount;
      if (batchDeleteEnabled) {
        try {
          deleteCount = deleteUnusedAssetBlobsBatch(assetBlobStore, format, contentStore);
        }
        finally {
          if (!batchDeleteExecutorService.isShutdown()) {
            batchDeleteExecutorService.shutdown();
          }
        }
      }
      else {
        deleteCount = deleteUnusedAssetBlobs(assetBlobStore, format, contentStore);
      }
      if (deleteCount > 0) {
        log.info("Deleted {} unused {} blobs from {}", deleteCount, format, contentStore);
      }
    }
    else {
      log.warn("Unknown format {}", format);
    }

    return null;
  }

  /**
   * Deletes unused {@link AssetBlob}s for the given format and content store.
   *
   * @return count of deleted asset blobs
   * @deprecated use {@link #deleteUnusedAssetBlobsBatch(AssetBlobStore, String, String)}} instead
   */
  @Deprecated
  private int deleteUnusedAssetBlobs(
      final AssetBlobStore<?> assetBlobStore,
      final String format,
      final String contentStore)
  {
    int deleteCount = 0;

    Continuation<AssetBlob> unusedAssetBlobs =
        assetBlobStore.browseUnusedAssetBlobs(BATCH_SIZE, BLOB_CREATED_DELAY_MINUTE, null);
    while (!isCanceled() && !unusedAssetBlobs.isEmpty()) {
      log.debug("Found {} unused {} blobs in {}", unusedAssetBlobs.size(), format, contentStore);
      for (AssetBlob assetBlob : unusedAssetBlobs) {
        if (isCanceled()) {
          break;
        }
        try {
          if (deleteAssetBlob(assetBlobStore, assetBlob.blobRef())) {
            deleteCount++;
          }
          else {
            // this doesn't necessarily indicate a problem, could be this particular blob is no longer unused
            log.debug("Could not delete {} blob {} from {}", format, assetBlob.blobRef(), contentStore);
          }
        }
        catch (RuntimeException e) {
          // this doesn't necessarily indicate a problem, could be this particular blob is no longer unused
          log.debug("Could not delete {} blob {} from {}", format, assetBlob.blobRef(), contentStore, e);
        }
      }
      unusedAssetBlobs = assetBlobStore.browseUnusedAssetBlobs(
          BATCH_SIZE, BLOB_CREATED_DELAY_MINUTE, unusedAssetBlobs.nextContinuationToken());
    }
    return deleteCount;
  }

  /**
   * Deletes unused {@link AssetBlob}s for the given format and content store in a batch manner.
   *
   * @return count of deleted asset blobs
   */
  @VisibleForTesting
  int deleteUnusedAssetBlobsBatch(
      final AssetBlobStore<?> assetBlobStore,
      final String format,
      final String contentStore)
  {
    int deleteCount = 0;
    Continuation<AssetBlob> unusedAssetBlobs =
        assetBlobStore.browseUnusedAssetBlobs(BATCH_SIZE, BLOB_CREATED_DELAY_MINUTE, null);
    while (!isCanceled() && !unusedAssetBlobs.isEmpty()) {
      if (isCanceled()) {
        break;
      }
      log.debug("Found {} unused {} blobs in {}", unusedAssetBlobs.size(), format, contentStore);
      List<BlobRef> blobRefAll = extractBlobRefsFromAssetBlobs(unusedAssetBlobs);
      List<BlobRef> successfullyDeletedBlobs = deleteAssetBlobsExecutorService(blobRefAll);
      if (!successfullyDeletedBlobs.isEmpty()) {
        deleteCount += deleteAssetBlobRecordsFromDatabase(assetBlobStore, successfullyDeletedBlobs);
      }

      unusedAssetBlobs = assetBlobStore.browseUnusedAssetBlobs(
          BATCH_SIZE, BLOB_CREATED_DELAY_MINUTE, unusedAssetBlobs.nextContinuationToken());
    }
    return deleteCount;
  }

  /**
   * Deletes the {@link AssetBlob} and associated {@link Blob}.
   *
   * @return {@code true} if the asset blob was deleted; otherwise {@code false}
   */
  private boolean deleteAssetBlob(final AssetBlobStore<?> assetBlobStore, final BlobRef blobRef) {
    boolean assetBlobDeleted = false;

    BlobStore blobStore = blobStoreManager.get(blobRef.getStore());
    if (blobStore == null) {
      // blobstore is deleted - delete the orphaned database record
      log.warn("Blob store {} no longer exists, deleting orphaned database record for {}",
          blobRef.getStore(), blobRef);
      assetBlobDeleted = assetBlobStore.deleteAssetBlob(blobRef);
    }
    else {
      assetBlobDeleted = assetBlobStore.deleteAssetBlob(blobRef);
      if (assetBlobDeleted && !deleteBlobContent(blobStore, blobRef)) {
        log.warn("Could not delete blob content under {}", blobRef);
        // still report asset blob as deleted...
      }
    }

    return assetBlobDeleted;
  }

  /**
   * Deletes batch of {@link Blob}s from their blob stores.
   *
   * @return list of blob refs that were successfully deleted or were already deleted
   */
  private List<BlobRef> deleteAssetBlobsExecutorService(final List<BlobRef> blobRefs) {
    ConcurrentLinkedQueue<BlobRef> successfullyDeleted = new ConcurrentLinkedQueue<>();
    CountDownLatch latch = new CountDownLatch(blobRefs.size());
    for (BlobRef blobRef : blobRefs) {
      batchDeleteExecutorService.submit(() -> {
        try {
          BlobStore blobStore = blobStoreManager.get(blobRef.getStore());
          if (blobStore == null) {
            // blobstore is deleted - mark for database record deletion
            log.warn("Blob store {} no longer exists, will delete orphaned database record for {}",
                blobRef.getStore(), blobRef);
            successfullyDeleted.add(blobRef);
          }
          else {
            if (!deleteBlobContent(blobStore, blobRef)) {
              log.warn(
                  "Blob content was already absent under {}; database record will still be removed",
                  blobRef);
            }
            successfullyDeleted.add(blobRef);
          }
        }
        catch (Exception e) {
          log.warn("Failed to delete blob content under {}", blobRef, e);
        }
        finally {
          latch.countDown();
        }
      });
    }
    try {
      latch.await();
    }
    catch (InterruptedException ex) {
      log.debug("CountDownLatch interrupted", ex);
    }
    return new ArrayList<>(successfullyDeleted);
  }

  /**
   * Deletes asset blob records from database for successfully deleted blobs.
   *
   * @param assetBlobStore the asset blob store to delete from
   * @param successfullyDeletedBlobs list of blobs that were successfully deleted from storage
   * @return number of records successfully deleted from database
   */
  private int deleteAssetBlobRecordsFromDatabase(
      final AssetBlobStore<?> assetBlobStore,
      final List<BlobRef> successfullyDeletedBlobs)
  {
    String[] blobRefIds = successfullyDeletedBlobs.stream()
        .map(BlobRefTypeHandler::toPersistableString)
        .toArray(String[]::new);

    try {
      int deletedCount = assetBlobStore.deleteAssetBlobBatch(blobRefIds);
      if (deletedCount < blobRefIds.length) {
        int missingCount = blobRefIds.length - deletedCount;
        log.warn("Batch processing: {} blobs deleted from storage, but only {} database records deleted " +
            "({} records not found)",
            blobRefIds.length, deletedCount, missingCount);
      }
      else {
        log.debug("Batch processing: {} blob records deleted from database", deletedCount);
      }
      return deletedCount;
    }
    catch (Exception e) {
      log.warn("Batch processing: {} blobs deleted from storage, but database deletion failed.",
          blobRefIds.length, e);
      return 0;
    }
  }

  /**
   * Deletes the {@link Blob} from its {@link BlobStore}.
   *
   * @return {@code true} if the asset blob was deleted; otherwise {@code false}
   */
  private boolean deleteBlobContent(BlobStore blobStore, final BlobRef blobRef) {
    if (HARD_DELETE) {
      return blobStore.deleteHard(blobRef.getBlobId());
    }
    boolean deleted = blobStore.delete(blobRef.getBlobId(), "Removing unused asset blob");
    if (!deleted) {
      blobStore.deleteHard(blobRef.getBlobId());
    }
    return deleted;
  }

  private List<BlobRef> extractBlobRefsFromAssetBlobs(final Continuation<AssetBlob> assetBlobs) {
    return assetBlobs.stream()
        .map(AssetBlob::blobRef)
        .collect(Collectors.toList());
  }

  @Override
  public String getMessage() {
    return getName();
  }

  @Override
  public void cancel() {
    super.cancel();
    if (batchDeleteExecutorService != null) {
      batchDeleteExecutorService.shutdown();
    }
  }
}
