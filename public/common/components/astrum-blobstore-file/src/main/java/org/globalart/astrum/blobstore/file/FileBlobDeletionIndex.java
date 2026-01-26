
package org.globalart.astrum.blobstore.file;

import java.io.IOException;

import org.globalart.astrum.blobstore.api.BlobId;
import org.globalart.astrum.blobstore.api.softdeleted.SoftDeletedBlobIndex;
import org.globalart.astrum.common.property.PropertiesFile;

/**
 * Storage holding {@link BlobId} of soft-deleted blobs for future hard-deletion
 */
public interface FileBlobDeletionIndex
    extends SoftDeletedBlobIndex
{
  /**
   * Initialisation of deletion index that should be done in the scope of {@link FileBlobStore} startup
   *
   * @param metadata blobstore-related properties
   * @param blobStore the {@link FileBlobStore} instance that this deletion index is related to
   */
  void initIndex(PropertiesFile metadata, FileBlobStore blobStore) throws IOException;

  /**
   * Tear down of deletion index that should be done in the scope of {@link FileBlobStore} tear down
   */
  void stopIndex() throws IOException;
}
