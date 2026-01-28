
package org.globalart.astrum.blobstore.internal.datastore;

import java.util.HashMap;
import java.util.Map;

import org.globalart.astrum.blobstore.api.Blob;
import org.globalart.astrum.blobstore.api.BlobId;
import org.globalart.astrum.blobstore.api.BlobRef;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.facet.ContentFacetStores;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.store.AssetBlobStore;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.test.util.Whitebox;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static java.util.Optional.empty;
import static java.util.Optional.of;
import static org.hamcrest.CoreMatchers.equalTo;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.blobstore.api.BlobStore.REPO_NAME_HEADER;

@RunWith(MockitoJUnitRunner.Silent.class)
public class DefaultBlobStoreUsageCheckerTest

{
  private static final String REPO_NAME = "repoName";

  private static final String NODE_ID = "repoName";

  private static final String DEFAULT = "default";

  private static final String NOT_DEFAULT = "notADefault";

  private static final String BLOB_ID_STRING = "86e20baa-0bca-4915-a7dc-9a4f34e72321";

  private static final BlobId BLOB_ID = new BlobId(BLOB_ID_STRING);

  private static final String BLOB_NAME = "/fake/blob.name";

  @Mock
  RepositoryManager repositoryManager;

  @Mock
  BlobStore blobStore;

  @Mock
  Repository repository;

  @Mock
  BlobStoreConfiguration blobStoreConfiguration;

  @Mock
  AssetBlobStore assetBlobStore;

  @Mock
  Blob blob;

  @Mock
  AssetBlob assetBlob;

  @Mock
  ContentFacetSupport contentFacet;

  @Mock
  ContentFacetStores contentFacetStores;

  DefaultBlobStoreUsageChecker underTest;

  @Before
  public void setUp() {
    Whitebox.setInternalState(contentFacetStores, "assetBlobStore", assetBlobStore);

    when(contentFacet.stores()).thenReturn(contentFacetStores);
    when(contentFacet.nodeName()).thenReturn(NODE_ID);

    BlobRef blobRef = new BlobRef(NODE_ID, DEFAULT, BLOB_ID.asUniqueString());

    when(blobStoreConfiguration.getName()).thenReturn(DEFAULT);

    when(blobStore.get(BLOB_ID)).thenReturn(blob);
    when(blobStore.getBlobStoreConfiguration()).thenReturn(blobStoreConfiguration);

    Map<String, String> headers = new HashMap<>();
    headers.put(REPO_NAME_HEADER, REPO_NAME);
    when(blob.getHeaders()).thenReturn(headers);

    when(repositoryManager.get(REPO_NAME)).thenReturn(repository);

    when(repository.facet(ContentFacet.class)).thenReturn(contentFacet);

    when(assetBlobStore.readAssetBlob(any())).thenReturn(empty());
    when(assetBlobStore.readAssetBlob(eq(blobRef))).thenReturn(of(assetBlob));

    underTest = new DefaultBlobStoreUsageChecker(repositoryManager);
  }

  @Test
  public void blobIsReferenced() {
    assertThat(underTest.test(blobStore, BLOB_ID, BLOB_NAME), equalTo(true));
  }

  @Test
  public void blobIdDoesNotMatch() {
    assertThat(underTest.test(blobStore, new BlobId("0"), BLOB_NAME), equalTo(false));
  }

  @Test
  public void blobStoreNameDoesNotMatch() {
    when(blobStoreConfiguration.getName()).thenReturn(NOT_DEFAULT);

    assertThat(underTest.test(blobStore, BLOB_ID, BLOB_NAME), equalTo(false));
  }
}
