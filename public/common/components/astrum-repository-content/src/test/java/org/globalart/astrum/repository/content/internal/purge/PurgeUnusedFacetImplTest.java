
package org.globalart.astrum.repository.content.internal.purge;

import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.facet.ContentFacetStores;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.store.AssetStore;
import org.globalart.astrum.repository.content.store.ComponentStore;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.mockito.ArgumentMatchers.anyInt;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class PurgeUnusedFacetImplTest
{
  @Mock
  private Repository repository;

  @Mock
  private ContentFacetSupport contentFacet;

  @Mock
  private ComponentStore<?> componentStore;

  @Mock
  private AssetStore<?> assetStore;

  @Mock
  private ContentFacetStores stores;

  @Mock
  private Configuration configuration;

  private PurgeUnusedFacetImpl underTest;

  @Before
  public void setUp() throws Exception {
    when(repository.facet(ContentFacet.class)).thenReturn(contentFacet);
    when(repository.getName()).thenReturn("test-repo");
    when(repository.getConfiguration()).thenReturn(configuration);

    stores = mock(ContentFacetStores.class, invocation -> {
      throw new UnsupportedOperationException();
    });

    // Use reflection to set the public fields on the mock
    ContentFacetStores realStores = mock(ContentFacetStores.class);
    when(contentFacet.stores()).thenReturn(realStores);

    // ContentFacetStores has public fields, so we need a real-ish mock
    // Let's use a different approach - mock the ContentFacetSupport to return stores with public fields
    when(contentFacet.contentRepositoryId()).thenReturn(42);

    underTest = new PurgeUnusedFacetImpl();
    underTest.installDependencies(mock(EventManager.class));
    underTest.attach(repository);
    underTest.init();
    underTest.start();
  }

  @Test
  public void purgeUnused_purgesAssetsAndComponents() throws Exception {
    ContentFacetStores mockStores = mockContentFacetStores();
    when(contentFacet.stores()).thenReturn(mockStores);
    when(contentFacet.contentRepositoryId()).thenReturn(42);

    when(assetStore.purgeNotRecentlyDownloaded(42, 10)).thenReturn(5);
    when(componentStore.purgeNotRecentlyDownloaded(42, 10)).thenReturn(3);

    underTest.purgeUnused(10);

    verify(assetStore).purgeNotRecentlyDownloaded(42, 10);
    verify(componentStore).purgeNotRecentlyDownloaded(42, 10);
  }

  @Test
  public void purgeUnused_withDifferentNumberOfDays() throws Exception {
    ContentFacetStores mockStores = mockContentFacetStores();
    when(contentFacet.stores()).thenReturn(mockStores);
    when(contentFacet.contentRepositoryId()).thenReturn(99);

    when(assetStore.purgeNotRecentlyDownloaded(99, 30)).thenReturn(0);
    when(componentStore.purgeNotRecentlyDownloaded(99, 30)).thenReturn(0);

    underTest.purgeUnused(30);

    verify(assetStore).purgeNotRecentlyDownloaded(99, 30);
    verify(componentStore).purgeNotRecentlyDownloaded(99, 30);
  }

  @Test
  public void purgeUnused_withNoPurgeableContent() throws Exception {
    ContentFacetStores mockStores = mockContentFacetStores();
    when(contentFacet.stores()).thenReturn(mockStores);
    when(contentFacet.contentRepositoryId()).thenReturn(1);

    when(assetStore.purgeNotRecentlyDownloaded(anyInt(), anyInt())).thenReturn(0);
    when(componentStore.purgeNotRecentlyDownloaded(anyInt(), anyInt())).thenReturn(0);

    underTest.purgeUnused(7);

    verify(assetStore).purgeNotRecentlyDownloaded(1, 7);
    verify(componentStore).purgeNotRecentlyDownloaded(1, 7);
  }

  @Test(expected = IllegalArgumentException.class)
  public void purgeUnused_rejectsZeroDays() {
    underTest.purgeUnused(0);
  }

  @Test(expected = IllegalArgumentException.class)
  public void purgeUnused_rejectsNegativeDays() {
    underTest.purgeUnused(-1);
  }

  @Test
  public void purgeUnused_passesContentRepositoryIdFromFacet() throws Exception {
    ContentFacetStores mockStores = mockContentFacetStores();
    when(contentFacet.stores()).thenReturn(mockStores);
    when(contentFacet.contentRepositoryId()).thenReturn(777);

    when(assetStore.purgeNotRecentlyDownloaded(eq(777), anyInt())).thenReturn(0);
    when(componentStore.purgeNotRecentlyDownloaded(eq(777), anyInt())).thenReturn(0);

    underTest.purgeUnused(1);

    verify(assetStore).purgeNotRecentlyDownloaded(eq(777), eq(1));
    verify(componentStore).purgeNotRecentlyDownloaded(eq(777), eq(1));
  }

  private ContentFacetStores mockContentFacetStores() {
    try {
      ContentFacetStores mockStores = mock(ContentFacetStores.class);
      java.lang.reflect.Field componentStoreField = ContentFacetStores.class.getDeclaredField("componentStore");
      componentStoreField.setAccessible(true);
      componentStoreField.set(mockStores, componentStore);

      java.lang.reflect.Field assetStoreField = ContentFacetStores.class.getDeclaredField("assetStore");
      assetStoreField.setAccessible(true);
      assetStoreField.set(mockStores, assetStore);

      return mockStores;
    }
    catch (Exception e) {
      throw new RuntimeException(e);
    }
  }
}
