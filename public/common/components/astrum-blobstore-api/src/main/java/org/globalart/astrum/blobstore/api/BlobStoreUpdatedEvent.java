

package org.globalart.astrum.blobstore.api;

/**
 * Emitted when a {@link BlobStore} has been updated.
 *
 * @since 3.15
 */
public class BlobStoreUpdatedEvent
    extends BlobStoreEvent
{
  public BlobStoreUpdatedEvent(final BlobStore blobStore) {
    super(blobStore);
  }
}
