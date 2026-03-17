
package org.globalart.astrum.blobstore.restore.raw.internal;

import java.io.IOException;
import java.util.Properties;

import javax.annotation.Nonnull;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.blobstore.api.Blob;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.restore.datastore.BaseRestoreBlobStrategy;
import org.globalart.astrum.blobstore.restore.datastore.DataStoreRestoreBlobData;
import org.globalart.astrum.common.log.DryRunPrefix;
import org.globalart.astrum.content.raw.RawContentFacet;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.manager.RepositoryManager;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * @since 3.29
 */
@Component
@Qualifier("raw")
public class RawRestoreBlobStrategy
    extends BaseRestoreBlobStrategy<DataStoreRestoreBlobData>
{
  private final RepositoryManager repositoryManager;

  @Autowired
  public RawRestoreBlobStrategy(
      final DryRunPrefix dryRunPrefix,
      final RepositoryManager repositoryManager)
  {
    super(dryRunPrefix);
    this.repositoryManager = checkNotNull(repositoryManager);
  }

  @Override
  protected boolean canAttemptRestore(@Nonnull final DataStoreRestoreBlobData data) {
    Repository repository = data.getRepository();

    if (repository.optionalFacet(RawContentFacet.class).isPresent()) {
      return true;
    }
    else {
      log.warn("Skipping as Raw Facet not found on repository: {}", repository.getName());
      return false;
    }
  }

  @Override
  protected void createAssetFromData(final DataStoreRestoreBlobData data) throws IOException {
    RawContentFacet rawContentFacet = data.getRepository().facet(RawContentFacet.class);
    rawContentFacet.put(data.getBlobName(), data.createDetachedPayload());
  }

  @Override
  protected String getAssetPath(@Nonnull final DataStoreRestoreBlobData data) {
    return data.getBlobName();
  }

  @Override
  protected DataStoreRestoreBlobData createRestoreData(
      final Properties properties,
      final Blob blob,
      final BlobStore blobStore)
  {
    return new DataStoreRestoreBlobData(blob, properties, blobStore, repositoryManager);
  }

  @Override
  protected boolean isComponentRequired(final DataStoreRestoreBlobData data) {
    return true;
  }

  @Override
  public void after(final boolean updateAssets, final Repository repository) {
    // no-op
  }
}
