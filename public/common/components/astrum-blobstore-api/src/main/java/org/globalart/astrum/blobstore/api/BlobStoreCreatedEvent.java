

package org.globalart.astrum.blobstore.api;

/**
 * Emitted when a {@link BlobStore} has been created.
 *
 * @since 3.1
 */
public class BlobStoreCreatedEvent
    extends BlobStoreEvent
{
  public BlobStoreCreatedEvent(final BlobStore blobStore) {
    super(blobStore);
  }
}
