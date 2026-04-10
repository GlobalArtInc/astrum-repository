
package org.globalart.astrum.repository.apt.datastore.internal.store;

import javax.annotation.Nullable;

import org.apache.ibatis.annotations.Param;
import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.store.AssetDAO;

/**
 * @since 3.31
 */
public interface AptAssetDAO
    extends AssetDAO
{
  /**
   * Browse Package index metadata assets for a given repository.
   *
   * @param repositoryId the repository ID
   * @param limit maximum number of assets to return
   * @param continuationToken optional token to continue from previous query
   * @param pathPattern the path pattern to match (e.g., "/dists/bionic/main/binary-%")
   * @return continuation of assets matching the criteria
   */
  Continuation<Asset> browsePackageIndexAssets(
      @Param("repositoryId") int repositoryId,
      @Param("limit") int limit,
      @Param("continuationToken") @Nullable String continuationToken,
      @Param("pathPattern") String pathPattern);
}
