
package org.globalart.astrum.blobstore.api;

public class BlobStoreStoppedEvent
    extends BlobStoreEvent
{
  public BlobStoreStoppedEvent(BlobStore blobStore) {
    super(blobStore);
  }
}
