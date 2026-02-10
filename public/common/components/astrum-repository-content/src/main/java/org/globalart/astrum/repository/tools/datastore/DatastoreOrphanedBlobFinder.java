
package org.globalart.astrum.repository.tools.datastore;

import java.util.Optional;
import java.util.function.Consumer;
import java.util.stream.Stream;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.blobstore.api.BlobAttributes;
import org.globalart.astrum.blobstore.api.BlobId;
import org.globalart.astrum.blobstore.api.BlobRef;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.tools.OrphanedBlobFinder;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;
import static org.apache.commons.lang3.StringUtils.isNotBlank;
import static org.globalart.astrum.blobstore.api.BlobStore.BLOB_NAME_HEADER;
import static org.globalart.astrum.blobstore.api.BlobStore.REPO_NAME_HEADER;
import static org.globalart.astrum.repository.config.ConfigurationConstants.BLOB_STORE_NAME;
import static org.globalart.astrum.repository.config.ConfigurationConstants.STORAGE;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * Detects orphaned blobs (i.e. nn-deleted blobs that exist in the blobstore but not the asset table)
 *
 * @since 3.25
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class DatastoreOrphanedBlobFinder
    implements OrphanedBlobFinder
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final RepositoryManager repositoryManager;

  private final BlobStoreManager blobStoreManager;

  @Autowired
  public DatastoreOrphanedBlobFinder(
      final RepositoryManager repositoryManager,
      final BlobStoreManager blobStoreManager)
  {
    this.repositoryManager = checkNotNull(repositoryManager);
    this.blobStoreManager = checkNotNull(blobStoreManager);
  }

  /**
   * Delete orphaned blobs for all repositories
   */
  @Override
  public void delete() {
    log.info("Starting delete of orphaned blobs for all known blob stores");

    blobStoreManager.browse().forEach(this::delete);

    log.info("Finished deleting orphaned blobs");
  }

  /**
   * Delete orphaned blobs associated with a given repository
   *
   * @param repository - where to look for orphaned blobs
   */
  @Override
  public void delete(final Repository repository) {
    log.info("Starting delete of orphaned blobs for {}", repository.getName());

    delete(getBlobStoreForRepository(repository));

    log.info("Finished deleting orphaned blobs for {}", repository.getName());
  }

  private void delete(final BlobStore blobStore) {
    detect(blobStore, blobId -> {
      log.info("Deleting orphaned blob {} from blobstore {}", blobId, blobStore.getBlobStoreConfiguration().getName());

      blobStore.deleteHard(blobId);
    });
  }

  /**
   * Look for orphaned blobs in a given repository and callback for each blobId found
   *
   * @param repository - where to look for orphaned blobs
   * @param handler - callback to handle an orphaned blob
   */
  @Override
  public void detect(final Repository repository, final Consumer<BlobId> handler) {
    validateRepositoryConfiguration(repository);

    detect(getBlobStoreForRepository(repository), handler);
  }

  private void detect(final BlobStore blobStore, final Consumer<BlobId> handler) {
    Stream<BlobId> blobIds = blobStore.getBlobIdStream();

    blobIds.forEach(id -> {
      BlobAttributes attributes = blobStore.getBlobAttributes(id);
      if (attributes != null) {
        checkIfOrphaned(handler, id, attributes);
      }
      else {
        log.warn("Skipping cleanup for blob {} because blob properties not found", id);
      }
    });
  }

  private void checkIfOrphaned(final Consumer<BlobId> handler, final BlobId id, final BlobAttributes attributes) {
    String repositoryName = attributes.getHeaders().get(REPO_NAME_HEADER);

    if (repositoryName != null) {
      String assetName = attributes.getHeaders().get(BLOB_NAME_HEADER);

      Repository repository = repositoryManager.get(repositoryName);
      if (repository == null) {
        log.debug("Blob {} considered orphaned because repository with name {} no longer exists", id.asUniqueString(),
            repositoryName);

        handler.accept(id);
      }
      else {
        findAssociatedAsset(assetName, repository).ifPresent(asset -> {
          BlobRef blobRef = asset.blob().map(AssetBlob::blobRef).orElse(null);
          if (blobRef != null && !blobRef.getBlobId().asUniqueString().equals(id.asUniqueString())) {
            if (!attributes.isDeleted()) {
              handler.accept(id);
            }
            else {
              log.debug("Blob {} in repository {} not considered orphaned because it is already marked soft-deleted",
                  id.asUniqueString(), repositoryName);
            }
          }
        });
      }
    }
  }

  private BlobStore getBlobStoreForRepository(final Repository repository) {
    String blobStoreName = (String) repository.getConfiguration()
        .getAttributes()
        .get(STORAGE)
        .get(BLOB_STORE_NAME);

    return blobStoreManager.get(blobStoreName);
  }

  private Optional<Asset> findAssociatedAsset(final String assetName, final Repository repository) {
    return repository.facet(ContentFacet.class).assets().path(assetName).find().map(a -> (Asset) a);
  }

  private void validateRepositoryConfiguration(final Repository repository) {
    checkArgument(repository.getConfiguration().getAttributes() != null,
        "Repository configuration not found " + repository.getName());
    checkArgument(repository.getConfiguration().getAttributes().get(STORAGE) != null,
        "No storage configuration found for the repository " + repository.getName());
    checkArgument(
        isNotBlank((String) repository.getConfiguration().getAttributes().get(STORAGE).get(BLOB_STORE_NAME)),
        "Blob store name not set for repository " + repository.getName());
  }
}
