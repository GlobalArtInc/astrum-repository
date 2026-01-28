
package org.globalart.astrum.blobstore.internal.datastore;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.blobstore.api.Blob;
import org.globalart.astrum.blobstore.api.BlobId;
import org.globalart.astrum.blobstore.api.BlobRef;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreUsageChecker;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.manager.RepositoryManager;

import com.codahale.metrics.annotation.Timed;
import org.springframework.context.annotation.Primary;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Optional.of;
import static org.globalart.astrum.blobstore.api.BlobStore.REPO_NAME_HEADER;

/**
 * Check if a blob is referenced in the corresponding metadata for NewDB
 *
 * @since 3.29
 */
@Primary
@Component
public class DefaultBlobStoreUsageChecker
    implements BlobStoreUsageChecker
{
  private final RepositoryManager repositoryManager;

  @Autowired
  public DefaultBlobStoreUsageChecker(final RepositoryManager repositoryManager) {
    this.repositoryManager = checkNotNull(repositoryManager);
  }

  @Override
  @Timed
  public boolean test(final BlobStore blobStore, final BlobId blobId, final String blobName) {
    return of(blobId)
        .map(blobStore::get)
        .map(Blob::getHeaders)
        .map(headers -> headers.get(REPO_NAME_HEADER))
        .map(repositoryManager::get)
        .map(repository -> (ContentFacetSupport) repository.facet(ContentFacet.class))
        .flatMap(contentFacetSupport -> {
          String blobStoreName = blobStore.getBlobStoreConfiguration().getName();
          BlobRef blobRef = new BlobRef(
              contentFacetSupport.nodeName(), blobStoreName, blobId.asUniqueString(), blobId.getBlobCreatedRef());
          return contentFacetSupport.stores().assetBlobStore.readAssetBlob(blobRef);
        })
        .isPresent();
  }
}
