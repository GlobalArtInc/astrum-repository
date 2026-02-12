
package org.globalart.astrum.repository.content.facet;

import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.repository.content.store.FormatStoreManager;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class ContentFacetStoresTest
{
  @Mock
  private BlobStoreManager blobStoreManager;

  @Mock
  private BlobStore blobStore;

  @Mock
  private FormatStoreManager formatStoreManager;

  @Before
  public void setUp() {
    when(blobStoreManager.get(anyString())).thenReturn(blobStore);
  }

  @Test
  public void testStoreCreation() {
    ContentFacetStores underTest = new ContentFacetStores(
        blobStoreManager, "default", formatStoreManager, "testStoreName");

    assertThat(underTest.blobStoreName, is("default"));
    assertThat(underTest.contentStoreName, is("testStoreName"));
    assertThat(underTest.blobStoreProvider, is(notNullValue()));
  }

  @Test
  public void testBlobStoreProviderLazyLoads() {
    ContentFacetStores underTest = new ContentFacetStores(
        blobStoreManager, "my-store", formatStoreManager, "content");

    BlobStore result = underTest.blobStoreProvider.get();
    assertThat(result, is(blobStore));
  }

  @Test
  public void testAssetStoreAccessor() {
    ContentFacetStores underTest = new ContentFacetStores(
        blobStoreManager, "default", formatStoreManager, "testStoreName");

    assertThat(underTest.assetStore(), is(underTest.assetStore));
  }

  @Test
  public void testAssetBlobStoreAccessor() {
    ContentFacetStores underTest = new ContentFacetStores(
        blobStoreManager, "default", formatStoreManager, "testStoreName");

    assertThat(underTest.assetBlobStore(), is(underTest.assetBlobStore));
  }
}
