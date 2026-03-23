

package org.globalart.astrum.content.maven.store;

import javax.annotation.Nullable;

import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.store.AssetDAO;

import org.apache.ibatis.annotations.Param;

/**
 * @since 3.25
 */
public interface Maven2AssetDAO
    extends AssetDAO
{
  /**
   * Find jar assets associated with Components in the namespace of kind maven-plugin.
   *
   * @param repositoryId the repository to search
   * @param limit maximum number of assets to return
   * @param continuationToken optional token to continue from a previous request
   * @param namespace the namespace to find plugins for
   * @return collection of assets and the next continuation token
   *
   * @see Continuation#nextContinuationToken()
   */
  Continuation<Asset> findMavenPluginAssetsForNamespace(
      @Param("repositoryId") int repositoryId,
      @Param("limit") int limit,
      @Nullable @Param("continuationToken") String continuationToken,
      @Param("namespace") String namespace);
}
