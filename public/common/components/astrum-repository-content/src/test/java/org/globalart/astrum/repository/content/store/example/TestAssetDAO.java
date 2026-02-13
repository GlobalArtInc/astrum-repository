
package org.globalart.astrum.repository.content.store.example;

import java.util.List;
import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.store.AssetDAO;

import org.apache.ibatis.annotations.Param;

/**
 * Test format which adds flagging of assets.
 */
public interface TestAssetDAO
    extends AssetDAO
{
  /**
   * Adds extra asset test columns to the owning data store.
   */
  void extendSchema();

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
  Continuation<Asset> browseFlaggedAssets(
      @Param("repositoryId") int repositoryId,
      @Param("limit") int limit,
      @Param("continuationToken") @Nullable String continuationToken);

  /**
   * Updates the test flag of the given asset in the content data store.
   *
   * @param asset the asset to update
   */
  void updateAssetFlag(TestAssetData asset);

  /**
   * Retrieves an asset located at the given path in the content data store.
   *
   * @param repositoryId the repository containing the asset
   * @param path the path of the asset
   * @return asset if it was found
   */
  Optional<TestAssetData> readPathTest(@Param("repositoryId") int repositoryId, @Param("path") String path);

  /**
   * Deletes assets by paths
   */
  void deleteAssetsByPaths(
      @Param("repositoryId") int repositoryId,
      @Param("paths") List<String> paths,
      @Param("updateComponentEntityVersion") boolean updateComponentEntityVersion);
}
