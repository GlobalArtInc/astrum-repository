

package org.globalart.astrum.blobstore.api;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * {@link BlobStore} event.
 *
 * @since 3.1
 */
public abstract class BlobStoreEvent
{
  private final BlobStore blobStore;

  public BlobStoreEvent(final BlobStore blobStore) {
    this.blobStore = checkNotNull(blobStore);
  }

  public BlobStore getBlobStore() {
    return blobStore;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "blobStore=" + blobStore +
        '}';
  }
}
