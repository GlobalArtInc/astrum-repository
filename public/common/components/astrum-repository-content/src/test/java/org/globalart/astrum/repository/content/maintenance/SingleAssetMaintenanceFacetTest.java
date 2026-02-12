
package org.globalart.astrum.repository.content.maintenance;

import java.util.Arrays;
import java.util.Optional;
import java.util.Set;

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
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class SingleAssetMaintenanceFacetTest
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

  private SingleAssetMaintenanceFacet underTest;

  @Before
  public void setUp() throws Exception {
    when(repository.facet(ContentFacet.class)).thenReturn(contentFacet);
    when(contentFacet.components()).thenReturn(fluentComponents);
    when(contentFacet.assets()).thenReturn(fluentAssets);

    FormatStoreManager formatStoreManager = mock(FormatStoreManager.class);
    when(formatStoreManager.componentStore(any())).thenReturn(componentStore);
    ContentFacetStores stores = new ContentFacetStores(blobStoreManager, "default", formatStoreManager, "astrum");
    when(contentFacet.stores()).thenReturn(stores);

    underTest = new SingleAssetMaintenanceFacet();
    underTest.attach(repository);
  }

  @Test
  public void deleteAsset_deletesComponentWhenAssetHasComponent() {
    Asset asset = mock(Asset.class);
    Component component = mock(Component.class);
    when(asset.component()).thenReturn(Optional.of(component));

    FluentComponent fluentComponent = mock(FluentComponent.class);
    when(fluentComponents.with(component)).thenReturn(fluentComponent);

    FluentAsset componentAsset = mock(FluentAsset.class);
    when(componentAsset.delete()).thenReturn(true);
    when(componentAsset.path()).thenReturn("/path/artifact.jar");
    when(fluentComponent.assets()).thenReturn(Arrays.asList(componentAsset));

    Set<String> deletedPaths = underTest.deleteAsset(asset);

    assertThat(deletedPaths, hasSize(1));
    assertThat(deletedPaths, containsInAnyOrder("/path/artifact.jar"));
    verify(fluentComponent).delete();
  }

  @Test
  public void deleteAsset_deletesOnlyAssetWhenNoComponent() {
    Asset asset = mock(Asset.class);
    when(asset.component()).thenReturn(Optional.empty());

    FluentAsset fluentAsset = mock(FluentAsset.class);
    when(fluentAssets.with(asset)).thenReturn(fluentAsset);
    when(fluentAsset.delete()).thenReturn(true);
    when(fluentAsset.path()).thenReturn("/path/orphan.jar");

    Set<String> deletedPaths = underTest.deleteAsset(asset);

    assertThat(deletedPaths, hasSize(1));
    assertThat(deletedPaths, containsInAnyOrder("/path/orphan.jar"));
    verify(fluentComponents, never()).with(any(Component.class));
  }

  @Test
  public void deleteAsset_returnsEmptyWhenAssetDeleteFails() {
    Asset asset = mock(Asset.class);
    when(asset.component()).thenReturn(Optional.empty());

    FluentAsset fluentAsset = mock(FluentAsset.class);
    when(fluentAssets.with(asset)).thenReturn(fluentAsset);
    when(fluentAsset.delete()).thenReturn(false);

    Set<String> deletedPaths = underTest.deleteAsset(asset);

    assertThat(deletedPaths, is(empty()));
  }
}
