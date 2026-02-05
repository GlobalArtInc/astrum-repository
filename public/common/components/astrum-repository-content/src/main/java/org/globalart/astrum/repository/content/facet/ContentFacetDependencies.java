
package org.globalart.astrum.repository.content.facet;

import java.util.Optional;
import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.common.node.NodeAccess;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.repository.move.RepositoryMoveService;
import org.globalart.astrum.repository.search.normalize.VersionNormalizerService;
import org.globalart.astrum.repository.storage.BlobMetadataStorage;
import org.globalart.astrum.security.ClientInfoProvider;
import org.globalart.astrum.validation.ConstraintViolationFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Internal dependencies injected into a repository's {@link ContentFacet}.
 *
 * @since 3.24
 */
@Component
public class ContentFacetDependencies
{
  private final BlobStoreManager blobStoreManager;

  private final DataSessionSupplier dataSessionSupplier;

  private final ConstraintViolationFactory constraintViolationFactory;

  private final ClientInfoProvider clientInfoProvider;

  private final NodeAccess nodeAccess;

  private final AssetBlobValidators assetBlobValidators;

  private final BlobMetadataStorage blobMetadataStorage;

  private final VersionNormalizerService versionNormalizerService;

  private final Optional<RepositoryMoveService> maybeMoveService;

  @Autowired
  public ContentFacetDependencies(
      final BlobStoreManager blobStoreManager,
      final DataSessionSupplier dataSessionSupplier,
      final ConstraintViolationFactory constraintViolationFactory,
      final ClientInfoProvider clientInfoProvider,
      final NodeAccess nodeAccess,
      final AssetBlobValidators assetBlobValidators,
      final BlobMetadataStorage blobMetadataStorage,
      final VersionNormalizerService versionNormalizerService,
      @Nullable final RepositoryMoveService moveService)
  {
    this.blobStoreManager = checkNotNull(blobStoreManager);
    this.dataSessionSupplier = checkNotNull(dataSessionSupplier);
    this.constraintViolationFactory = checkNotNull(constraintViolationFactory);
    this.clientInfoProvider = checkNotNull(clientInfoProvider);
    this.nodeAccess = checkNotNull(nodeAccess);
    this.assetBlobValidators = checkNotNull(assetBlobValidators);
    this.blobMetadataStorage = checkNotNull(blobMetadataStorage);
    this.versionNormalizerService = versionNormalizerService;
    this.maybeMoveService = Optional.ofNullable(moveService);
  }

  public BlobStoreManager getBlobStoreManager() {
    return blobStoreManager;
  }

  public DataSessionSupplier getDataSessionSupplier() {
    return dataSessionSupplier;
  }

  public ConstraintViolationFactory getConstraintViolationFactory() {
    return constraintViolationFactory;
  }

  public ClientInfoProvider getClientInfoProvider() {
    return clientInfoProvider;
  }

  public NodeAccess getNodeAccess() {
    return nodeAccess;
  }

  public AssetBlobValidators getAssetBlobValidators() {
    return assetBlobValidators;
  }

  public BlobMetadataStorage getBlobMetadataStorage() {
    return blobMetadataStorage;
  }

  public VersionNormalizerService getVersionNormalizerService() {
    return versionNormalizerService;
  }

  public Optional<RepositoryMoveService> getMoveService() {
    return maybeMoveService;
  }
}
