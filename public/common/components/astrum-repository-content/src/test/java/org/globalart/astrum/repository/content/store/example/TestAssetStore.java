
package org.globalart.astrum.repository.content.store.example;

import javax.annotation.Nullable;

import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.store.AssetStore;
import org.globalart.astrum.transaction.Transactional;

import org.springframework.beans.factory.annotation.Autowired;

/**
 * Enhanced test asset store.
 */
public class TestAssetStore
    extends AssetStore<TestAssetDAO>
{
  @Autowired
  public TestAssetStore(
      final DataSessionSupplier sessionSupplier,
      final String storeName)
  {
    super(sessionSupplier, false, storeName, TestAssetDAO.class);
  }

  /**
   * Browse all flagged assets in the given repository in a paged fashion.
   *
   * @param repositoryId the repository to browse
   * @param limit maximum number of assets to return
   * @param continuationToken optional token to continue from a previous request
   * @return collection of assets and the next continuation token
   *
   * @see Continuation#nextContinuationToken()
   */
  @Transactional
  public Continuation<Asset> browseFlaggedAssets(
      final int repositoryId,
      final int limit,
      @Nullable final String continuationToken)
  {
    return dao().browseFlaggedAssets(repositoryId, limit, continuationToken);
  }

  /**
   * Updates the test flag in the given asset.
   *
   * @param asset the asset to update
   */
  @Transactional
  public void updateAssetFlag(final TestAssetData asset) {
    dao().updateAssetFlag(asset);
  }
}
