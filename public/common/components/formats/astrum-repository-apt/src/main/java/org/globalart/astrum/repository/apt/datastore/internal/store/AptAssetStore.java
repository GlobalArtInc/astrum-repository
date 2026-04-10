
package org.globalart.astrum.repository.apt.datastore.internal.store;

import javax.annotation.Nullable;

import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.store.AssetStore;
import org.globalart.astrum.transaction.Transactional;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;

import static org.globalart.astrum.common.app.FeatureFlags.DATASTORE_CLUSTERED_ENABLED_NAMED_VALUE;

/**
 * APT asset store.
 */
public class AptAssetStore
    extends AssetStore<AptAssetDAO>
{
  @Autowired
  public AptAssetStore(
      final DataSessionSupplier sessionSupplier,
      @Value(DATASTORE_CLUSTERED_ENABLED_NAMED_VALUE) final boolean clustered,
      final String storeName)
  {
    super(sessionSupplier, clustered, storeName, AptAssetDAO.class);
  }

  /**
   * Browse Package index metadata assets for a given repository.
   *
   * @param repositoryId the repository ID
   * @param limit maximum number of assets to return
   * @param continuationToken optional token to continue from previous query
   * @param pathPattern the path pattern to match (e.g., "/dists/bionic/main/binary-%")
   * @return continuation of assets matching the criteria
   */
  @Transactional
  public Continuation<Asset> browsePackageIndexAssets(
      final int repositoryId,
      final int limit,
      @Nullable final String continuationToken,
      final String pathPattern)
  {
    return dao().browsePackageIndexAssets(repositoryId, limit, continuationToken, pathPattern);
  }
}
