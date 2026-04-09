
package org.globalart.astrum.blobstore.restore.apt.internal;

import java.io.IOException;
import java.util.Optional;
import java.util.Properties;

import javax.annotation.Nonnull;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.blobstore.api.Blob;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.restore.datastore.BaseRestoreBlobStrategy;
import org.globalart.astrum.blobstore.restore.datastore.DataStoreRestoreBlobData;
import org.globalart.astrum.common.log.DryRunPrefix;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.apt.AptFormat;
import org.globalart.astrum.repository.apt.datastore.AptContentFacet;
import org.globalart.astrum.repository.apt.debian.Utils;
import org.globalart.astrum.repository.manager.RepositoryManager;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

/**
 * @since 3.31
 */
@Component
@Qualifier(AptFormat.NAME)
public class AptRestoreBlobStrategy
    extends BaseRestoreBlobStrategy<DataStoreRestoreBlobData>
{
  private final RepositoryManager repositoryManager;

  @Autowired
  public AptRestoreBlobStrategy(
      final DryRunPrefix dryRunPrefix,
      final RepositoryManager repositoryManager)
  {
    super(dryRunPrefix);
    this.repositoryManager = repositoryManager;
  }

  @Override
  protected boolean canAttemptRestore(
      @Nonnull final DataStoreRestoreBlobData data)
  {
    Repository repository = data.getRepository();

    Optional<AptContentFacet> facet = repository.optionalFacet(AptContentFacet.class);
    if (!facet.isPresent()) {
      log.warn("Skipping as {} not found on repository: {}", facet.getClass().getSimpleName(), repository.getName());
      return false;
    }
    return true;
  }

  @Override
  protected void createAssetFromData(
      final DataStoreRestoreBlobData data) throws IOException
  {
    String assetPath = getAssetPath(data);
    data.getRepository()
        .facet(AptContentFacet.class)
        .put(assetPath, data.createDetachedPayload());
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
    String blobName = data.getBlobName();
    return Utils.isDebPackageContentType(blobName);
  }

  @Override
  public void after(final boolean updateAssets, final Repository repository) {
    // no-op
  }
}
