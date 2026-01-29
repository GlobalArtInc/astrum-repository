
package org.globalart.astrum.repository.internal.blobstore;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreLocator;
import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.blobstore.api.RepositoryDoesNotExistException;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.manager.RepositoryManager;

import static org.globalart.astrum.repository.config.ConfigurationConstants.BLOB_STORE_NAME;
import static org.globalart.astrum.repository.config.ConfigurationConstants.STORAGE;
import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Default {@link BlobStoreLocator} implementation.
 *
 * @since 3.34
 */
@Component
public class BlobStoreLocatorImpl
    implements BlobStoreLocator
{

  private final BlobStoreManager blobStoreManager;

  private final RepositoryManager repositoryManager;

  @Autowired
  public BlobStoreLocatorImpl(final BlobStoreManager blobStoreManager, final RepositoryManager repositoryManager) {
    this.blobStoreManager = checkNotNull(blobStoreManager);
    this.repositoryManager = checkNotNull(repositoryManager);
  }

  @Override
  public BlobStore getBlobStore(final String repositoryName) throws RepositoryDoesNotExistException {
    Repository repository = repositoryManager.get(repositoryName);
    if (repository == null) {
      throw new RepositoryDoesNotExistException();
    }
    Configuration configuration = repository.getConfiguration();
    String blobStoreName = configuration.attributes(STORAGE).require(BLOB_STORE_NAME, String.class);
    return blobStoreManager.get(blobStoreName);
  }
}
