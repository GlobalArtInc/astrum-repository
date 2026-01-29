
package org.globalart.astrum.repository.replication;

import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.repository.replication.ReplicationBlobStoreStatusManager.BlobStoreReplicationStatus;

import org.springframework.context.annotation.Primary;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.replication.ReplicationBlobStoreStatusManager.BlobStoreReplicationStatus.ATTRIBUTES_MISSING;

/**
 * @since 3.32
 */
@Primary
@Component
public class DefaultReplicationBlobStoreStatusManager
    implements ReplicationBlobStoreStatusManager
{

  @Override
  public BlobStoreReplicationStatus getReplicationStatus(final BlobStoreConfiguration configuration) {
    return ATTRIBUTES_MISSING;
  }

  @Override
  public void initializeReplicationStatus(final BlobStoreConfiguration configuration) {
    // no-op
  }

  @Override
  public void setReplicationStatus(
      final BlobStoreConfiguration configuration,
      final BlobStoreReplicationStatus status)
  {
    throw new UnsupportedOperationException();
  }
}
