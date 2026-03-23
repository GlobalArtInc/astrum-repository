
package org.globalart.astrum.content.maven.store;

import javax.annotation.Nullable;

import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.store.AssetStore;
import org.globalart.astrum.transaction.Transactional;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;

import static org.globalart.astrum.common.app.FeatureFlags.DATASTORE_CLUSTERED_ENABLED_NAMED_VALUE;

public class Maven2AssetStore
    extends AssetStore<Maven2AssetDAO>
{
  @Autowired
  public Maven2AssetStore(
      final DataSessionSupplier sessionSupplier,
      @Value(DATASTORE_CLUSTERED_ENABLED_NAMED_VALUE) final boolean clustered,
      final String storeName)
  {
    super(sessionSupplier, clustered, storeName, Maven2AssetDAO.class);
  }

  @Transactional
  public Continuation<Asset> findMavenPluginAssetsForNamespace(
      final int repositoryId,
      final int limit,
      @Nullable final String continuationToken,
      final String namespace)
  {
    return dao().findMavenPluginAssetsForNamespace(repositoryId, limit, continuationToken, namespace);
  }
}
