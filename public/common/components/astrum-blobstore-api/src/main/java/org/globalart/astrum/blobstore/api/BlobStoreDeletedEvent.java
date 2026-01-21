

package org.globalart.astrum.blobstore.api;

/**
 * Emitted when a {@link BlobStore} has been deleted.
 *
 * @since 3.1
 */
public class BlobStoreDeletedEvent
    extends BlobStoreEvent
{
  public BlobStoreDeletedEvent(final BlobStore blobStore) {
    super(blobStore);
  }
}
