
package org.globalart.astrum.blobstore.api.softdeleted;

import java.time.OffsetDateTime;
import java.util.List;
import java.util.stream.Stream;

import org.globalart.astrum.blobstore.api.BlobId;
import org.globalart.astrum.common.entity.Continuation;

/**
 * Store for accessing the soft deleted blob ID's
 */
public interface SoftDeletedBlobsStore
{
  /**
   * Create new record
   *
   * @param blobId string representation of {@link BlobId}
   * @param sourceBlobStoreName the blobstore name this record is related to
   */
  void createRecord(BlobId blobId, String sourceBlobStoreName);

  /**
   * Return all records stored in DB, the continuationToken to be used when amount more than single page (>1000 rows)
   *
   * @param continuationToken the record id used for pagination
   * @param sourceBlobStoreName the blobstore name these records are related to
   * @return all records related to provided sourceBlobStoreName
   */
  Continuation<SoftDeletedBlob> readRecords(String continuationToken, int limit, String sourceBlobStoreName);

  Stream<BlobId> readAllBlobIds(String sourceBlobStoreName);

  Stream<BlobId> getBlobsBefore(final String sourceBlobStoreName, OffsetDateTime blobsDeletedBefore);

  /**
   * Delete single record by provided 'blobId' related to specified blobstore name
   *
   * @param sourceBlobStoreName the blobstore name this record is related to
   * @param blobId {@link BlobId} of record that should be deleted
   */
  void deleteRecord(String sourceBlobStoreName, BlobId blobId);

  /**
   * Delete all records related to provided blobstore name
   *
   * @param sourceBlobStoreName the blobstore name these records are related to
   */
  void deleteAllRecords(String sourceBlobStoreName);

  /**
   * Returns the amount of soft deleted blobs records related to provided blobstore
   *
   * @param sourceBlobStoreName the blobstore name these records are related to
   * @return amount of soft deleted blobs
   */
  int count(String sourceBlobStoreName);

  /**
   * Returns the amount of soft deleted blobs records related to provided blobstore
   *
   * @param sourceBlobStoreName the blobstore name these records are related to
   * @param blobsDeletedBefore an upper bound of when blobs were deleted
   * @return amount of soft deleted blobs
   */
  int countBefore(String blobStoreName, OffsetDateTime blobsDeletedBefore);

  /**
   * Batch update the blob location (source_blob_store_name and date_path_ref) for multiple soft deleted blob records.
   * Each blob can have different target blob store names and date path references.
   *
   * @param updates list of {@link BlobLocationUpdate} records containing blobId, newSourceBlobStoreName, and
   *          newDatePathRef
   * @param oldSourceBlobStoreName the current/old blob store name in the records
   * @return number of rows updated
   */
  int batchUpdateBlobLocation(List<BlobLocationUpdate> updates, String oldSourceBlobStoreName);
}
