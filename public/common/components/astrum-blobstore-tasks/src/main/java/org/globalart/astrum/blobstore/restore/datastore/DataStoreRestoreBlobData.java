
package org.globalart.astrum.blobstore.restore.datastore;

import java.util.Properties;

import org.globalart.astrum.blobstore.api.Blob;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.restore.RestoreBlobData;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.view.payloads.DetachedBlobPayload;

/**
 * Simple structure for relevant data for a blob during metadata restoration
 *
 * @since 3.38
 */
public class DataStoreRestoreBlobData
    extends RestoreBlobData
{
  public DataStoreRestoreBlobData(
      final Blob blob,
      final Properties blobProperties,
      final BlobStore blobStore,
      final RepositoryManager repositoryManager)
  {
    super(blob, blobProperties, blobStore, repositoryManager);
  }

  @Override
  public String getBlobName() {
    String blobName = super.getBlobName();
    if (blobName.startsWith("/")) {
      return blobName;
    }
    return "/" + blobName;
  }

  /**
   * Creates a {@link DetachedBlobPayload} that includes the source blobstore reference.
   * This allows the ingest process to detect when a blob needs to be re-ingested into a different blobstore.
   */
  public DetachedBlobPayload createDetachedPayload() {
    return new DetachedBlobPayload(getBlob(), getBlobStore());
  }
}
