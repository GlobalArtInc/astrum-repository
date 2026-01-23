
package org.globalart.astrum.blobstore.internal;

import java.time.OffsetDateTime;
import java.util.Objects;
import java.util.stream.Stream;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.blobstore.api.BlobId;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.softdeleted.SoftDeletedBlobIndex;
import org.globalart.astrum.blobstore.api.softdeleted.SoftDeletedBlobsStore;
import org.globalart.astrum.common.stateguard.Guarded;
import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;

import org.springframework.core.Ordered;
import org.springframework.context.annotation.Primary;
import org.springframework.core.annotation.Order;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Preconditions.checkState;
import static org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport.State.STARTED;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

@Primary
@Component
@Qualifier("default")
@Order(Ordered.HIGHEST_PRECEDENCE)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class SoftDeletedBlobIndexImpl
    extends StateGuardLifecycleSupport
    implements SoftDeletedBlobIndex
{
  protected final SoftDeletedBlobsStore softDeletedBlobsStore;

  protected String blobStoreName;

  @Autowired
  public SoftDeletedBlobIndexImpl(final SoftDeletedBlobsStore softDeletedBlobsStore) {
    this.softDeletedBlobsStore = checkNotNull(softDeletedBlobsStore);
  }

  @Override
  public void init(final BlobStore blobStore) {
    checkNotNull(blobStore);
    String newBlobStoreName = checkNotNull(blobStore.getBlobStoreConfiguration().getName());
    checkState(blobStoreName == null || Objects.equals(newBlobStoreName, blobStoreName), "Previously initialized");

    if (blobStoreName != null) {
      return;
    }

    blobStoreName = newBlobStoreName;
    try {
      start();
    }
    catch (Exception e) {
      if (e instanceof RuntimeException re) {
        throw re;
      }
      throw new IllegalArgumentException(e);
    }
  }

  @Guarded(by = STARTED)
  @Override
  public final void createRecord(final BlobId blobId) {
    softDeletedBlobsStore.createRecord(blobId, blobStoreName);
  }

  @Guarded(by = STARTED)
  @Override
  public final Stream<BlobId> getRecordsBefore(final OffsetDateTime blobsOlderThan) {
    return softDeletedBlobsStore.getBlobsBefore(blobStoreName, blobsOlderThan);
  }

  @Guarded(by = STARTED)
  @Override
  public final void deleteRecord(final BlobId blobId) {
    softDeletedBlobsStore.deleteRecord(blobStoreName, blobId);
  }

  @Guarded(by = STARTED)
  @Override
  public final void deleteAllRecords() {
    softDeletedBlobsStore.deleteAllRecords(blobStoreName);
  }

  @Guarded(by = STARTED)
  @Override
  public final int size() {
    return softDeletedBlobsStore.count(blobStoreName);
  }

  @Guarded(by = STARTED)
  @Override
  public int count(final OffsetDateTime blobsBefore) {
    return softDeletedBlobsStore.countBefore(blobStoreName, blobsBefore);
  }
}
