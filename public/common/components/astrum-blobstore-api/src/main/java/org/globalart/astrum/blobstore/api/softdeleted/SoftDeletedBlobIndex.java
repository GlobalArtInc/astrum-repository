
package org.globalart.astrum.blobstore.api.softdeleted;

import java.time.OffsetDateTime;
import java.util.stream.Stream;

import org.globalart.astrum.blobstore.api.BlobId;
import org.globalart.astrum.blobstore.api.BlobStore;

/**
 * An index of soft-deleted blobs for a blobstore
 */
public interface SoftDeletedBlobIndex
{
  /**
   * Initializes the index for the provided blobstore
   */
  void init(BlobStore blobstore);

  /**
   * Add new record to deletion index
   *
   * @param blobId the {@link BlobId} that is referenced to the Blob that was soft-deleted
   */
  void createRecord(BlobId blobId);

  /**
   * Deletes specified record by {@link BlobId}
   *
   * @param blobId the {@link BlobId} of record to be deleted
   */
  void deleteRecord(BlobId blobId);

  /**
   * Deletes all records currently holding
   */
  void deleteAllRecords();

  /**
   * @return a stream of blobids associated with the initialized blobstore that were deleted prior to the provided date
   */
  Stream<BlobId> getRecordsBefore(OffsetDateTime blobsBefore);

  /**
   * Returns the amount of soft deleted blobs records
   *
   * @return amount of soft deleted blobs
   */
  int size();

  /**
   * Returns the amount of soft deleted blobs records
   *
   * @return amount of soft deleted blobs
   */
  int count(OffsetDateTime blobsBefore);
}
