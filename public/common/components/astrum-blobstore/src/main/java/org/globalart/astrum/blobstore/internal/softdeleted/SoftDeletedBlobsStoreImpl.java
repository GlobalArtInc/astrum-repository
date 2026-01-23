
package org.globalart.astrum.blobstore.internal.softdeleted;

import java.time.OffsetDateTime;
import java.util.List;
import java.util.stream.Stream;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.blobstore.api.BlobId;
import org.globalart.astrum.blobstore.api.softdeleted.BlobLocationUpdate;
import org.globalart.astrum.blobstore.api.softdeleted.SoftDeletedBlob;
import org.globalart.astrum.blobstore.api.softdeleted.SoftDeletedBlobsStore;
import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.common.entity.Continuations;
import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.transaction.Transactional;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Implementation of {@link SoftDeletedBlobsStore}
 */
@Component
@Qualifier("mybatis")
public class SoftDeletedBlobsStoreImpl
    extends ConfigStoreSupport<SoftDeletedBlobsDAO>
    implements SoftDeletedBlobsStore
{
  @Autowired
  public SoftDeletedBlobsStoreImpl(final DataSessionSupplier sessionSupplier) {
    super(sessionSupplier);
  }

  @Transactional
  @Override
  public void createRecord(final BlobId blobId, final String sourceBlobStoreName) {
    dao().createRecord(sourceBlobStoreName, blobId.toString(), blobId.getBlobCreatedRef());
  }

  @Transactional
  @Override
  public Continuation<SoftDeletedBlob> readRecords(
      final String continuationToken,
      final int limit,
      final String sourceBlobStoreName)
  {
    return dao().readRecords(continuationToken, limit, sourceBlobStoreName);
  }

  @Transactional
  public Continuation<SoftDeletedBlob> readRecordsBefore(
      final String continuationToken,
      final int limit,
      final String sourceBlobStoreName,
      final OffsetDateTime upperBound)
  {
    return dao().readRecordsBefore(continuationToken, limit, sourceBlobStoreName, upperBound);
  }

  @Override
  public Stream<BlobId> readAllBlobIds(final String sourceBlobStoreName) {
    return Continuations
        .streamOf((limit, continuationToken) -> readRecords(continuationToken, limit, sourceBlobStoreName))
        .map(data -> new BlobId(data.getBlobId(), data.getDatePathRef()));
  }

  @Override
  public Stream<BlobId> getBlobsBefore(final String sourceBlobStoreName, final OffsetDateTime blobsDeletedBefore) {
    return Continuations
        .streamOf((limit, token) -> readRecordsBefore(token, limit, sourceBlobStoreName, blobsDeletedBefore))
        .map(data -> new BlobId(data.getBlobId(), data.getDatePathRef()));
  }

  @Transactional
  @Override
  public void deleteRecord(final String sourceBlobStoreName, final BlobId blobId) {
    dao().deleteRecord(sourceBlobStoreName, blobId.toString());
  }

  @Override
  public void deleteAllRecords(final String sourceBlobStoreName) {
    while (doDeleteAllBlobs(sourceBlobStoreName) != 0) {
      log.trace("Deleted page for {}", sourceBlobStoreName);
    }
  }

  @Transactional
  public int doDeleteAllBlobs(final String sourceBlobName) {
    return dao().deleteAllRecords(sourceBlobName, "1000");
  }

  @Transactional
  @Override
  public int count(final String sourceBlobStoreName) {
    return dao().count(sourceBlobStoreName);
  }

  @Transactional
  @Override
  public int countBefore(final String blobStoreName, final OffsetDateTime blobsDeletedBefore) {
    return dao().countBefore(blobStoreName, blobsDeletedBefore);
  }

  @Transactional
  @Override
  public int batchUpdateBlobLocation(final List<BlobLocationUpdate> updates, final String oldSourceBlobStoreName) {
    if (updates.isEmpty()) {
      return 0;
    }
    return dao().batchUpdateBlobLocation(updates, oldSourceBlobStoreName);
  }
}
