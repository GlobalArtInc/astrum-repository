
package org.globalart.astrum.blobstore.internal.softdeleted;

import java.time.OffsetDateTime;
import java.util.List;

import javax.annotation.Nullable;

import org.globalart.astrum.blobstore.api.BlobId;
import org.globalart.astrum.blobstore.api.softdeleted.BlobLocationUpdate;
import org.globalart.astrum.blobstore.api.softdeleted.SoftDeletedBlob;
import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.datastore.api.DataAccess;

import org.apache.ibatis.annotations.Param;

/**
 * {@link SoftDeletedBlob} DataAccess
 */
public interface SoftDeletedBlobsDAO
    extends DataAccess
{
  /**
   * Create new record
   *
   * @param sourceBlobStoreName the blobstore name this record is related to
   * @param blobId string representation of {@link BlobId}
   * @param datePathRef the {@link OffsetDateTime} of the blob creation
   */
  void createRecord(
      @Param("sourceBlobStoreName") String sourceBlobStoreName,
      @Param("blobId") String blobId,
      @Param("datePathRef") OffsetDateTime datePathRef);

  /**
   * Return all records stored in DB, the continuationToken to be used when amount more than single page (>1000 rows)
   *
   * @param continuationToken the record id used for pagination
   * @param sourceBlobStoreName the blobstore name these records are related to
   * @return all records related to provided sourceBlobStoreName
   */
  Continuation<SoftDeletedBlob> readRecords(
      @Nullable @Param("continuationToken") String continuationToken,
      @Param("limit") int limit,
      @Param("sourceBlobStoreName") String sourceBlobStoreName);

  Continuation<SoftDeletedBlob> readRecordsBefore(
      @Nullable @Param("continuationToken") String continuationToken,
      @Param("limit") int limit,
      @Param("sourceBlobStoreName") String sourceBlobStoreName,
      @Param("blobsDeletedBefore") OffsetDateTime upperBound);

  /**
   * Delete single record by provided 'blobId' related to specified blobstore name
   *
   * @param sourceBlobStoreName the blobstore name this record is related to
   * @param blobId {@link BlobId} of record that should be deleted
   */
  void deleteRecord(
      @Param("sourceBlobStoreName") String sourceBlobStoreName,
      @Param("blobId") String blobId);

  /**
   * Delete all records related to provided blobstore name
   *
   * @param sourceBlobStoreName the blobstore name these records are related to
   * @param limit maximum amount of rows to be deleted
   * @return numbers of deleted rows
   */
  int deleteAllRecords(
      @Param("sourceBlobStoreName") String sourceBlobStoreName,
      @Param("limit") String limit);

  /**
   * Returns the amount of soft deleted blobs records related to provided blobstore
   *
   * @param sourceBlobStoreName the blobstore name these records are related to
   * @return amount of soft deleted blobs
   */
  int count(@Param("sourceBlobStoreName") String sourceBlobStoreName);

  /**
   * Returns the amount of soft deleted blobs records related to provided blobstore
   *
   * @param sourceBlobStoreName the blobstore name these records are related to
   * @param blobsDeletedBefore an upper bound of when blobs were deleted
   * @return amount of soft deleted blobs
   */
  int countBefore(
      @Param("sourceBlobStoreName") String blobStoreName,
      @Param("blobsDeletedBefore") OffsetDateTime blobsDeletedBefore);

  /**
   * Batch update the blob location (source_blob_store_name and date_path_ref) for multiple soft deleted blob records.
   * Each blob can be updated with different values for newSourceBlobStoreName and newDatePathRef.
   *
   * @param updates list of {@link BlobLocationUpdate} objects containing blobId, newSourceBlobStoreName, and
   *          newDatePathRef
   * @param oldSourceBlobStoreName the current/old blob store name in the records
   * @return number of rows updated
   */
  int batchUpdateBlobLocation(
      @Param("updates") List<BlobLocationUpdate> updates,
      @Param("oldSourceBlobStoreName") String oldSourceBlobStoreName);
}
