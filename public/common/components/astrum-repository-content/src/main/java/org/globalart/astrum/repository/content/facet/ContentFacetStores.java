
package org.globalart.astrum.repository.content.facet;

import jakarta.inject.Provider;

import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.repository.content.store.AssetBlobStore;
import org.globalart.astrum.repository.content.store.AssetStore;
import org.globalart.astrum.repository.content.store.ComponentStore;
import org.globalart.astrum.repository.content.store.ContentRepositoryStore;
import org.globalart.astrum.repository.content.store.FormatStoreManager;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Persistence stores associated with a repository's {@link ContentFacet}.
 *
 * @since 3.24
 */
public class ContentFacetStores
{
  public final String blobStoreName;

  public final Provider<BlobStore> blobStoreProvider;

  public final String contentStoreName;

  public final ContentRepositoryStore<?> contentRepositoryStore;

  public final ComponentStore<?> componentStore;

  public final AssetStore<?> assetStore;

  public final AssetBlobStore<?> assetBlobStore;

  public ContentFacetStores(
      final BlobStoreManager blobStoreManager,
      final String blobStoreName,
      final FormatStoreManager formatStoreManager,
      final String contentStoreName)
  {
    this.blobStoreName = checkNotNull(blobStoreName);
    // We have to use Provider here because the blobstore may be not initialized after conversion to group.
    this.blobStoreProvider = () -> blobStoreManager.get(blobStoreName);

    this.contentStoreName = checkNotNull(contentStoreName);
    this.contentRepositoryStore = formatStoreManager.contentRepositoryStore(contentStoreName);
    this.componentStore = formatStoreManager.componentStore(contentStoreName);
    this.assetStore = formatStoreManager.assetStore(contentStoreName);
    this.assetBlobStore = formatStoreManager.assetBlobStore(contentStoreName);
  }

  public AssetStore<?> assetStore() {
    return assetStore;
  }

  public AssetBlobStore<?> assetBlobStore() {
    return assetBlobStore;
  }
}
