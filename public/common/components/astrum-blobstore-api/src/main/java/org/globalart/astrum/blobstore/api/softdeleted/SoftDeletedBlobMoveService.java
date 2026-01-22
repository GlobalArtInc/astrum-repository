
package org.globalart.astrum.blobstore.api.softdeleted;

import java.util.Map;

import org.globalart.astrum.blobstore.api.BlobId;

/**
 * Service for updating soft_deleted_blobs records after blobs are moved between blob stores.
 * <p>
 * When blobs are moved (e.g., during group member removal or repository move operations),
 * their soft_deleted_blobs records need to be updated with:
 * <ul>
 * <li>The new source_blob_store_name where the blob now resides</li>
 * <li>The new date_path_ref timestamp (assigned during move by BlobStoreSupport.getBlobId())</li>
 * </ul>
 */
public interface SoftDeletedBlobMoveService
{
  /**
   * Updates soft_deleted_blobs records after blobs are moved to new locations.
   * Updates both source_blob_store_name and date_path_ref for each moved blob.
   *
   * @param movedBlobs map of moved blob IDs (containing new datePathRef) to their target blob store names
   * @param oldBlobStoreName the blob store the blobs were moved from
   */
  void updateMovedBlobRecords(Map<BlobId, String> movedBlobs, String oldBlobStoreName);
}
