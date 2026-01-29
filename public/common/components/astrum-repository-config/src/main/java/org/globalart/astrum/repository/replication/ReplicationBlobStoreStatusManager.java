
package org.globalart.astrum.repository.replication;

import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;

public interface ReplicationBlobStoreStatusManager
{

  BlobStoreReplicationStatus getReplicationStatus(BlobStoreConfiguration configuration);

  void initializeReplicationStatus(BlobStoreConfiguration configuration);

  void setReplicationStatus(BlobStoreConfiguration configuration, BlobStoreReplicationStatus status);

  public enum BlobStoreReplicationStatus
  {
    ATTRIBUTES_MISSING,
    ATTRIBUTES_UPGRADING,
    ATTRIBUTES_PRESENT
  }
}
