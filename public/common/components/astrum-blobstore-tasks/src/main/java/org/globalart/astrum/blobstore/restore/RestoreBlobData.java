
package org.globalart.astrum.blobstore.restore;

import java.time.OffsetDateTime;
import java.util.Properties;

import org.globalart.astrum.blobstore.api.Blob;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.manager.RepositoryManager;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.blobstore.api.BlobAttributesConstants.HEADER_PREFIX;
import static org.globalart.astrum.blobstore.api.BlobStore.BLOB_NAME_HEADER;
import static org.globalart.astrum.blobstore.api.BlobStore.CONTENT_TYPE_HEADER;
import static org.globalart.astrum.blobstore.api.BlobStore.REPO_NAME_HEADER;

/**
 * Simple structure for relevant data for a blob during metadata restoration
 *
 * @since 3.6.1
 */
public abstract class RestoreBlobData
{
  private final Blob blob;

  private final Properties blobProperties;

  private final BlobStore blobStore;

  private final Repository repository;

  private OffsetDateTime lastDownloaded;

  protected RestoreBlobData(
      final Blob blob,
      final Properties blobProperties,
      final BlobStore blobStore,
      final RepositoryManager repositoryManager)
  {
    checkNotNull(repositoryManager);
    checkNotNull(blobProperties);

    this.blob = blob;
    this.blobProperties = blobProperties;
    this.blobStore = blobStore;
    this.repository = repositoryManager
        .get(checkNotNull(getProperty(HEADER_PREFIX + REPO_NAME_HEADER), "Blob properties missing repository name"));
  }

  public Blob getBlob() {
    return blob;
  }

  public String getBlobName() {
    return getProperty(HEADER_PREFIX + BLOB_NAME_HEADER);
  }

  public String getBlobType() {
    return getProperty(HEADER_PREFIX + CONTENT_TYPE_HEADER);
  }

  public BlobStore getBlobStore() {
    return blobStore;
  }

  public Repository getRepository() {
    return repository;
  }

  public final String getProperty(final String propertyName) {
    return blobProperties.getProperty(propertyName);
  }

  public void setLastDownloaded(final OffsetDateTime lastDownloaded) {
    this.lastDownloaded = lastDownloaded;
  }

  public OffsetDateTime getLastDownloaded() {
    return lastDownloaded;
  }

  public boolean hasLastDownloaded() {
    return lastDownloaded != null;
  }
}
