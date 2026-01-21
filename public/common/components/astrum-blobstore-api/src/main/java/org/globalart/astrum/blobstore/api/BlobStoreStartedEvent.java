

package org.globalart.astrum.blobstore.api;

public class BlobStoreStartedEvent
    extends BlobStoreEvent
{
  public BlobStoreStartedEvent(BlobStore blobStore) {
    super(blobStore);
  }
}
