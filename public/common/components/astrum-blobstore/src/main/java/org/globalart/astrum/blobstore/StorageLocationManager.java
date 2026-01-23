
package org.globalart.astrum.blobstore;

import java.io.IOException;

import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;

/**
 * Manages a storage location.
 *
 * @since 3.16
 */
public interface StorageLocationManager
{
  /**
   * Prepares the storage location at start up. Should initialize the storage location if needed, or validate it if
   * already created.
   */
  void prepareStorageLocation(BlobStoreConfiguration blobStoreConfiguration) throws IOException;

  /**
   * Deletes the storage location when the blob store is removed.
   */
  void deleteStorageLocation(BlobStoreConfiguration blobStoreConfiguration) throws IOException;
}
