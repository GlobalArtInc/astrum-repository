
package org.globalart.astrum.repository.content.facet;

import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.common.node.NodeAccess;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.repository.move.RepositoryMoveService;
import org.globalart.astrum.repository.search.normalize.VersionNormalizerService;
import org.globalart.astrum.repository.storage.BlobMetadataStorage;
import org.globalart.astrum.validation.ConstraintViolationFactory;

import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

@RunWith(MockitoJUnitRunner.Silent.class)
public class ContentFacetDependenciesTest
{
  @Mock
  private BlobStoreManager blobStoreManager;

  @Mock
  private DataSessionSupplier dataSessionSupplier;

  @Mock
  private ConstraintViolationFactory constraintViolationFactory;

  @Mock
  private org.globalart.astrum.security.ClientInfoProvider clientInfoProvider;

  @Mock
  private NodeAccess nodeAccess;

  @Mock
  private AssetBlobValidators assetBlobValidators;

  @Mock
  private BlobMetadataStorage blobMetadataStorage;

  @Mock
  private VersionNormalizerService versionNormalizerService;

  @Mock
  private RepositoryMoveService moveService;

  @Test
  public void testConstructorWithAllDependencies() {
    ContentFacetDependencies underTest = new ContentFacetDependencies(
        blobStoreManager, dataSessionSupplier, constraintViolationFactory,
        clientInfoProvider, nodeAccess, assetBlobValidators, blobMetadataStorage,
        versionNormalizerService, moveService);

    assertThat(underTest.getBlobStoreManager(), is(blobStoreManager));
    assertThat(underTest.getDataSessionSupplier(), is(dataSessionSupplier));
    assertThat(underTest.getConstraintViolationFactory(), is(constraintViolationFactory));
    assertThat(underTest.getClientInfoProvider(), is(clientInfoProvider));
    assertThat(underTest.getNodeAccess(), is(nodeAccess));
    assertThat(underTest.getAssetBlobValidators(), is(assetBlobValidators));
    assertThat(underTest.getBlobMetadataStorage(), is(blobMetadataStorage));
    assertThat(underTest.getVersionNormalizerService(), is(versionNormalizerService));
    assertThat(underTest.getMoveService().isPresent(), is(true));
    assertThat(underTest.getMoveService().get(), is(moveService));
  }

  @Test
  public void testConstructorWithNullMoveService() {
    ContentFacetDependencies underTest = new ContentFacetDependencies(
        blobStoreManager, dataSessionSupplier, constraintViolationFactory,
        clientInfoProvider, nodeAccess, assetBlobValidators, blobMetadataStorage,
        versionNormalizerService, null);

    assertThat(underTest.getMoveService().isPresent(), is(false));
  }
}
