
package org.globalart.astrum.repository.content.maintenance;

import java.util.Arrays;
import java.util.Collections;
import java.util.Set;
import java.util.stream.Stream;

import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.facet.ContentFacetStores;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentAssets;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.fluent.FluentComponents;
import org.globalart.astrum.repository.content.store.ComponentStore;
import org.globalart.astrum.repository.content.store.FormatStoreManager;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.Matchers.empty;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyList;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class DefaultMaintenanceFacetTest
{
  @Mock
  private Repository repository;

  @Mock
  private ContentFacetSupport contentFacet;

  @Mock
  private FluentComponents fluentComponents;

  @Mock
  private FluentAssets fluentAssets;

  @Mock
  private BlobStoreManager blobStoreManager;

  @Mock
  private ComponentStore<?> componentStore;

  private DefaultMaintenanceFacet underTest;

  @Before
  public void setUp() throws Exception {
    when(repository.facet(ContentFacet.class)).thenReturn(contentFacet);
    when(contentFacet.components()).thenReturn(fluentComponents);
    when(contentFacet.assets()).thenReturn(fluentAssets);

    FormatStoreManager formatStoreManager = mock(FormatStoreManager.class);
    when(formatStoreManager.componentStore(any())).thenReturn(componentStore);
    ContentFacetStores stores = new ContentFacetStores(blobStoreManager, "default", formatStoreManager, "astrum");
    when(contentFacet.stores()).thenReturn(stores);

    underTest = new DefaultMaintenanceFacet();
    underTest.attach(repository);
  }

  // --- deleteComponent tests ---

  @Test
  public void deleteComponent_deletesAllAssetsAndComponent() {
    Component component = mock(Component.class);
    FluentComponent fluentComponent = mock(FluentComponent.class);
    when(fluentComponents.with(component)).thenReturn(fluentComponent);

    FluentAsset asset1 = mock(FluentAsset.class);
    when(asset1.delete()).thenReturn(true);
    when(asset1.path()).thenReturn("/path/asset1.jar");

    FluentAsset asset2 = mock(FluentAsset.class);
    when(asset2.delete()).thenReturn(true);
    when(asset2.path()).thenReturn("/path/asset2.pom");

    when(fluentComponent.assets()).thenReturn(Arrays.asList(asset1, asset2));

    Set<String> deletedPaths = underTest.deleteComponent(component);

    assertThat(deletedPaths, hasSize(2));
    assertThat(deletedPaths, containsInAnyOrder("/path/asset1.jar", "/path/asset2.pom"));
    verify(fluentComponent).delete();
  }

  @Test
  public void deleteComponent_onlyReportsPathsForSuccessfullyDeletedAssets() {
    Component component = mock(Component.class);
    FluentComponent fluentComponent = mock(FluentComponent.class);
    when(fluentComponents.with(component)).thenReturn(fluentComponent);

    FluentAsset successfullyDeleted = mock(FluentAsset.class);
    when(successfullyDeleted.delete()).thenReturn(true);
    when(successfullyDeleted.path()).thenReturn("/path/deleted.jar");

    FluentAsset alreadyDeleted = mock(FluentAsset.class);
    when(alreadyDeleted.delete()).thenReturn(false);

    when(fluentComponent.assets()).thenReturn(Arrays.asList(successfullyDeleted, alreadyDeleted));

    Set<String> deletedPaths = underTest.deleteComponent(component);

    assertThat(deletedPaths, hasSize(1));
    assertThat(deletedPaths, containsInAnyOrder("/path/deleted.jar"));
    // Component delete is still called even if some assets were not deleted by us
    verify(fluentComponent).delete();
  }

  @Test
  public void deleteComponent_returnsEmptySetWhenNoAssetsDeletedByUs() {
    Component component = mock(Component.class);
    FluentComponent fluentComponent = mock(FluentComponent.class);
    when(fluentComponents.with(component)).thenReturn(fluentComponent);

    FluentAsset asset = mock(FluentAsset.class);
    when(asset.delete()).thenReturn(false);

    when(fluentComponent.assets()).thenReturn(Collections.singletonList(asset));

    Set<String> deletedPaths = underTest.deleteComponent(component);

    assertThat(deletedPaths, is(empty()));
    verify(fluentComponent).delete();
  }

  @Test
  public void deleteComponent_handlesComponentWithNoAssets() {
    Component component = mock(Component.class);
    FluentComponent fluentComponent = mock(FluentComponent.class);
    when(fluentComponents.with(component)).thenReturn(fluentComponent);
    when(fluentComponent.assets()).thenReturn(Collections.emptyList());

    Set<String> deletedPaths = underTest.deleteComponent(component);

    assertThat(deletedPaths, is(empty()));
    verify(fluentComponent).delete();
  }

  // --- deleteAsset tests ---

  @Test
  public void deleteAsset_returnsPathWhenDeleteSucceeds() {
    Asset asset = mock(Asset.class);
    FluentAsset fluentAsset = mock(FluentAsset.class);
    when(fluentAssets.with(asset)).thenReturn(fluentAsset);
    when(fluentAsset.delete()).thenReturn(true);
    when(fluentAsset.path()).thenReturn("/content/artifact.jar");

    Set<String> deletedPaths = underTest.deleteAsset(asset);

    assertThat(deletedPaths, hasSize(1));
    assertThat(deletedPaths, containsInAnyOrder("/content/artifact.jar"));
  }

  @Test
  public void deleteAsset_returnsEmptySetWhenDeleteReturnsFalse() {
    Asset asset = mock(Asset.class);
    FluentAsset fluentAsset = mock(FluentAsset.class);
    when(fluentAssets.with(asset)).thenReturn(fluentAsset);
    when(fluentAsset.delete()).thenReturn(false);

    Set<String> deletedPaths = underTest.deleteAsset(asset);

    assertThat(deletedPaths, is(empty()));
  }

  // --- deleteComponents tests ---

  @Test
  public void deleteComponents_delegatesToComponentStorePurge() {
    FluentComponent comp1 = mock(FluentComponent.class);
    FluentComponent comp2 = mock(FluentComponent.class);

    when(contentFacet.contentRepositoryId()).thenReturn(42);
    when(componentStore.purge(eq(42), anyList())).thenReturn(2);

    int purged = underTest.deleteComponents(Stream.of(comp1, comp2));

    assertThat(purged, is(2));
    verify(componentStore).purge(eq(42), anyList());
  }

  @Test
  public void deleteComponents_returnsZeroWhenStreamIsEmpty() {
    when(contentFacet.contentRepositoryId()).thenReturn(42);
    when(componentStore.purge(eq(42), anyList())).thenReturn(0);

    int purged = underTest.deleteComponents(Stream.empty());

    assertThat(purged, is(0));
    verify(componentStore).purge(eq(42), anyList());
  }

  @Test
  public void deleteComponents_passesContentRepositoryIdFromFacet() {
    when(contentFacet.contentRepositoryId()).thenReturn(99);
    when(componentStore.purge(eq(99), anyList())).thenReturn(0);

    underTest.deleteComponents(Stream.empty());

    verify(componentStore).purge(eq(99), anyList());
  }
}
