
package org.globalart.astrum.repository.content.store;

import java.time.OffsetDateTime;
import java.util.Collection;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.blobstore.api.BlobRef;
import org.globalart.astrum.blobstore.api.ExternalMetadata;
import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.content.AssetReconcileData;
import org.globalart.astrum.transaction.Transactional;

import org.springframework.beans.factory.annotation.Autowired;
import org.apache.ibatis.annotations.Param;

/**
 * {@link AssetBlob} store.
 *
 * @since 3.21
 */
public class AssetBlobStore<T extends AssetBlobDAO>
    extends ContentStoreSupport<T>
{
  @Autowired
  public AssetBlobStore(
      final DataSessionSupplier sessionSupplier,
      final String contentStoreName,
      final Class<T> daoClass)
  {
    super(sessionSupplier, contentStoreName, daoClass);
  }

  /**
   * Browse unused asset blobs in the content data store in a paged fashion.
   *
   * @param limit maximum number of asset blobs to return
   * @param continuationToken optional token to continue from a previous request
   * @return collection of asset blobs and the next continuation token
   *
   * @see Continuation#nextContinuationToken()
   */
  @Transactional
  public Continuation<AssetBlob> browseUnusedAssetBlobs(
      final int limit,
      final int blobCreatedDelayMinute,
      @Nullable final String continuationToken)
  {
    return dao().browseUnusedAssetBlobs(limit, blobCreatedDelayMinute, continuationToken);
  }

  /**
   * Browse asset blobs in the content data store in a paged fashion.
   *
   * @param limit maximum number of asset blobs to return
   * @param continuationToken optional token to continue from a previous request
   * @return collection of asset blobs and the next continuation token
   *
   * @see Continuation#nextContinuationToken()
   */
  @Transactional
  public Continuation<AssetBlob> browseAssetBlobs(final int limit, @Nullable final String continuationToken) {
    return dao().browseAssetBlobs(limit, continuationToken);
  }

  /**
   * Browse asset blobs in the content data store in a paged fashion which are stored in the specified blobstore.
   *
   * @param limit maximum number of asset blobs to return
   * @param blobStoreName the blob store name
   * @param continuationToken optional token to continue from a previous request
   * @return collection of asset blobs and the next continuation token
   *
   * @see Continuation#nextContinuationToken()
   */
  @Transactional
  public Continuation<AssetBlob> browseAssetBlobsByBlobStore(
      final int limit,
      final String blobStoreName,
      @Nullable final String continuationToken)
  {
    return dao().browseAssetBlobsByBlobStore(limit, blobStoreName, continuationToken);
  }

  /**
   * Browse asset blobs in the content data store in a paged fashion by provided date range.
   *
   * @param limit maximum number of asset blobs to return
   * @param continuationToken optional token to continue from a previous request
   * @return collection of asset blobs and the next continuation token
   *
   * @see Continuation#nextContinuationToken()
   */
  @Transactional
  public Continuation<AssetReconcileData> browseAssetBlobsWithinDuration(
      final int limit,
      final OffsetDateTime start,
      final OffsetDateTime end,
      @Nullable final String continuationToken)
  {
    return dao().browseAssetBlobsWithinDuration(limit, start, end, continuationToken);
  }

  /**
   * Return count of asset blobs matched with provided date range
   */
  @Transactional
  public int countAssetBlobsWithinDuration(final OffsetDateTime start, final OffsetDateTime end) {
    return dao().countAssetBlobsWithinDuration(start, end);
  }

  /**
   * Creates the given asset blob in the content data store.
   *
   * @param assetBlob the asset blob to create
   */
  @Transactional
  public void createAssetBlob(final AssetBlobData assetBlob) {
    dao().createAssetBlob(assetBlob);
  }

  /**
   * Retrieves an asset blob from the content data store.
   *
   * @param blobRef the blob reference
   * @return asset blob if it was found
   */
  @Transactional
  public Optional<AssetBlob> readAssetBlob(final BlobRef blobRef) {
    return dao().readAssetBlob(blobRef);
  }

  /**
   * Deletes an asset blob from the content data store.
   *
   * @param blobRef the blob reference
   * @return {@code true} if the asset blob was deleted
   */
  @Transactional
  public boolean deleteAssetBlob(final BlobRef blobRef) {
    return dao().deleteAssetBlob(blobRef);
  }

  /**
   * Deletes batch of asset blobs from the content data store.
   *
   * @param blobRefIds the array of String with blobRefs
   * @return number of asset blobs deleted from database
   */
  @Transactional
  public int deleteAssetBlobBatch(final String[] blobRefIds) {
    return dao().deleteAssetBlobBatch(blobRefIds);
  }

  /**
   * Deletes an asset blob along with asset.
   *
   * @param blobRef the blob reference
   */
  @Transactional
  public void deleteAssetBlobWithAsset(final BlobRef blobRef) {
    dao().deleteAssetBlobWithAsset(blobRef);
  }

  /**
   * Generally it is recommended that this method not be called and let stores manage this value.
   *
   * @since 3.29
   */
  @Transactional
  public void setBlobCreated(final AssetBlob blob, final OffsetDateTime blobCreated) {
    dao().setBlobCreated(blob.blobRef(), blobCreated);
  }

  @Transactional
  public void setAddedToRepository(final AssetBlob blob, final OffsetDateTime addedToRepository) {
    dao().setAddedToRepository(blob.blobRef(), addedToRepository);
  }

  @Transactional
  public void setContentType(final AssetBlob blob, final String contentType) {
    dao().setContentType(blob.blobRef(), contentType);
  }

  @Transactional
  public void setChecksums(final AssetBlob blob, final Map<String, String> checksums) {
    dao().setChecksums(blob.blobRef(), checksums);
  }

  /**
   * Generally it is recommended that this method not be called and let stores manage this value.
   *
   * Sets the name of user who uploaded the asset.
   */
  @Transactional
  public void setCreatedBy(final AssetBlob blob, final String createdBy) {
    dao().setCreatedBy(blob.blobRef(), createdBy);
  }

  /**
   * Generally it is recommended that this method not be called and let stores manage this value.
   *
   * Sets the IP address of user who uploaded the asset.
   */
  @Transactional
  public void setCreatedByIP(final AssetBlob blob, final String createdByIP) {
    dao().setCreatedByIP(blob.blobRef(), createdByIP);
  }

  /**
   */
  @Transactional
  public void setExternalMetadata(final AssetBlob blob, final ExternalMetadata metadata) {
    dao().setExternalMetadata(blob.blobRef(), metadata);
  }

  /**
   * Browse asset blobs with legacy blobRef format {@code store-name:blob-id@node-id} in a paged fashion.
   *
   * @param limit maximum number of asset blobs to return
   * @param continuationToken optional token to continue from a previous request
   * @return collection of asset blobs and the next continuation token
   */
  @Transactional
  public Continuation<AssetBlob> browseAssetsWithLegacyBlobRef(
      final int limit,
      @Nullable final String continuationToken)
  {
    return dao().browseAssetsWithLegacyBlobRef(limit, continuationToken);
  }

  /**
   * Update asset blobs in a batch fashion.
   *
   * @param assetBlobs asset blobs for update
   * @return {code true} if asset blobs were updated
   */
  @Transactional
  public boolean updateBlobRefs(@Param("assetBlobs") final Collection<AssetBlob> assetBlobs) {
    return dao().updateBlobRefs(assetBlobs);
  }

  /**
   * Update asset blob.
   *
   * @param assetBlob asset blob for update
   * @return {code true} if asset blob was updated
   */
  @Transactional
  public boolean updateBlobRef(@Param("assetBlobData") final AssetBlob assetBlob) {
    return dao().updateBlobRef(assetBlob);
  }

  /**
   * Check asset blobs with legacy blobRef format {@code store-name:blob-id@node-id} exists for
   * given format.
   *
   * @return {@code true} if legacy blob refs exists
   */
  @Transactional
  public boolean notMigratedAssetBlobRefsExists() {
    return dao().existsNotMigratedAssetBlobs();
  }

  /**
   * Get repository name by blob reference.
   *
   * @param blobRef the blob reference
   * @return the repository name
   */
  @Transactional
  public String getRepositoryName(final BlobRef blobRef) {
    return dao().getRepositoryName(blobRef);
  }

  /**
   * Get path by blob reference.
   *
   * @param blobRef the blob reference
   * @return the path
   */
  @Transactional
  public String getPathByBlobRef(final BlobRef blobRef) {
    return dao().getPathByBlobRef(blobRef);
  }

  @Transactional
  public List<AssetReconcileData> browseAssetBlobsByBlobRefs(final List<BlobRef> blobRefs) {
    return dao().browseAssetBlobsByBlobRefs(blobRefs);
  }

  /**
   * Retrieves an asset blob from the content data store based on provided data.
   * By blobRef or (path and repository) if new version was uploaded
   *
   * @param blobRef the blob reference
   * @param path the blob path
   * @param repository the blob repository
   * @return asset blob if it was found
   */
  @Transactional
  public Optional<AssetBlob> readAssetBlobByPathAndRepository(
      final BlobRef blobRef,
      final String path,
      final String repository)
  {
    return dao().readAssetBlobByPathAndRepository(blobRef, path, repository);
  }
}
