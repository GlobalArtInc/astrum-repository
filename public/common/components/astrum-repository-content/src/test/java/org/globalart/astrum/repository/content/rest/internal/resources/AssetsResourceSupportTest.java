
package org.globalart.astrum.repository.content.rest.internal.resources;

import java.util.List;
import java.util.Optional;

import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.downloads.AssetDownloadCounter;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.facet.ContentFacetDependencies;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentAssets;
import org.globalart.astrum.repository.content.fluent.internal.FluentAssetImpl;
import org.globalart.astrum.repository.content.fluent.FluentContinuation;
import org.globalart.astrum.repository.content.maintenance.MaintenanceService;
import org.globalart.astrum.repository.content.store.AssetData;
import org.globalart.astrum.repository.move.RepositoryMoveService;
import org.globalart.astrum.repository.rest.api.RepositoryManagerRESTAdapter;
import org.globalart.astrum.repository.selector.ContentAuthHelper;
import org.globalart.astrum.repository.types.HostedType;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static java.util.Arrays.asList;
import static java.util.stream.IntStream.range;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.collection.IsCollectionWithSize.hasSize;
import static org.hamcrest.collection.IsEmptyCollection.empty;
import static org.hamcrest.core.Is.is;
import static org.hamcrest.core.IsNull.notNullValue;
import static org.hamcrest.core.IsNull.nullValue;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.repository.content.rest.internal.resources.AssetsResourceSupport.PAGE_SIZE_LIMIT;
import static org.globalart.astrum.repository.content.rest.internal.resources.AssetsResourceSupport.toInternalToken;
import static org.globalart.astrum.repository.content.rest.internal.resources.AssetsResourceSupport.trim;

@RunWith(MockitoJUnitRunner.Silent.class)
public class AssetsResourceSupportTest
{
  private static final String A_FORMAT = "A_Format";

  // tests assume we've stored more assets than the page limit
  private static final int NUMBER_OF_ASSETS = PAGE_SIZE_LIMIT + 2;

  private static final String ASSET_PATH = "/junit/junit/4.12/junit-4.12.jar";

  private static final String REPOSITORY_NAME = "repository1";

  private static final String REPOSITORY_URL = "http://localhost:8081/repository/" + REPOSITORY_NAME;

  private static final int ASSET_ID = 1001;

  private static final int A_REPOSITORY_ID = 1;

  @Mock
  private ContentFacetSupport contentFacetSupport;

  @Mock
  private Repository repository;

  @Mock
  private ContentFacet contentFacet;

  @Mock
  private ContentFacetDependencies dependencies;

  @Mock
  private ContentAuthHelper contentAuthHelper;

  @Mock
  private FluentAssets fluentAssets;

  @Mock
  private Format aFormat;

  @Mock
  private Continuation<FluentAsset> assetContinuation;

  @Mock
  private RepositoryManagerRESTAdapter repositoryManagerRESTAdapter;

  @Mock
  private MaintenanceService maintenanceService;

  private AssetsResource underTest;

  @Before
  public void setup() {
    mockRepository();
    mockContentFacet();
    mockFluentAssets();
    underTest = new AssetsResource(repositoryManagerRESTAdapter, maintenanceService, contentAuthHelper, null,
        mock(AssetDownloadCounter.class));
  }

  @Test
  public void browseShouldBeEmptyWhenNoAssets() {
    when(assetContinuation.isEmpty()).thenReturn(true);

    List<FluentAsset> assets = underTest.browse(repository, null);

    assertThat(assets, empty());
    verify(fluentAssets).browse(PAGE_SIZE_LIMIT, null);
    verify(contentAuthHelper, never()).checkPathPermissions(ASSET_PATH, A_FORMAT, REPOSITORY_NAME);
  }

  @Test
  public void browseShouldBeEmptyWhenNoPermittedAssets() {
    when(assetContinuation.isEmpty()).thenReturn(false).thenReturn(true);
    when(contentAuthHelper.checkPathPermissions(ASSET_PATH, A_FORMAT, REPOSITORY_NAME)).thenReturn(false);

    List<FluentAsset> assets = underTest.browse(repository, null);

    assertThat(assets, empty());
    verify(fluentAssets, times(2)).browse(PAGE_SIZE_LIMIT, null);
    verify(contentAuthHelper, times(NUMBER_OF_ASSETS)).checkPathPermissions(ASSET_PATH, A_FORMAT, REPOSITORY_NAME);
  }

  @Test
  public void browseShouldReturnPermittedAssets() {
    int numberOfAssetsNotPermitted = 4;
    int numberOfPermittedAssets = NUMBER_OF_ASSETS - numberOfAssetsNotPermitted;

    when(assetContinuation.isEmpty()).thenReturn(false).thenReturn(true);
    when(contentAuthHelper.checkPathPermissions(ASSET_PATH, A_FORMAT, REPOSITORY_NAME))
        .thenReturn(false)
        .thenReturn(false, false, false, true);

    List<FluentAsset> assets = underTest.browse(repository, null);

    assertThat(assets, hasSize(numberOfPermittedAssets));
    verify(fluentAssets, times(2)).browse(PAGE_SIZE_LIMIT, null);
    verify(contentAuthHelper, times(NUMBER_OF_ASSETS)).checkPathPermissions(ASSET_PATH, A_FORMAT, REPOSITORY_NAME);
  }

  @Test
  public void shouldTrimNumberOfAssetsToLimit() {
    when(assetContinuation.isEmpty()).thenReturn(false);
    when(contentAuthHelper.checkPathPermissions(ASSET_PATH, A_FORMAT, REPOSITORY_NAME)).thenReturn(true);

    List<FluentAsset> assets = underTest.browse(repository, null);

    assertThat(assets, hasSize(AssetsResource.PAGE_SIZE_LIMIT));
    verify(contentAuthHelper, times(NUMBER_OF_ASSETS)).checkPathPermissions(ASSET_PATH, A_FORMAT, REPOSITORY_NAME);
  }

  @Test
  public void trimShouldTrimToSizeSpecified() {
    List<String> list = asList("foo", "bar", "foobar");

    assertThat(trim(list, 2), is(list.subList(0, 2)));
  }

  @Test
  public void trimShouldReturnSameList() {
    List<String> list = asList("foo", "bar", "foobar");

    assertThat(trim(list, 3), is(list));
  }

  @Test
  public void shouldBeReturnInternalToken() {

    assertThat(toInternalToken(null), nullValue());

    assertThat(toInternalToken("123"), notNullValue());
  }

  private void mockRepository() {
    when(repository.facet(ContentFacet.class)).thenReturn(contentFacet);
    when(repository.getName()).thenReturn(REPOSITORY_NAME);
    when(repository.getUrl()).thenReturn(REPOSITORY_URL);
    when(repository.getFormat()).thenReturn(aFormat);
    when(repository.getType()).thenReturn(new HostedType());
    when(aFormat.getValue()).thenReturn(A_FORMAT);
  }

  private void mockContentFacet() {
    when(contentFacet.assets()).thenReturn(fluentAssets);
    when(contentFacetSupport.dependencies()).thenReturn(dependencies);
    when(dependencies.getMoveService()).thenReturn(Optional.of(mock(RepositoryMoveService.class)));
    when(contentFacet.contentRepositoryId()).thenReturn(A_REPOSITORY_ID);
  }

  private void mockFluentAssets() {
    when(fluentAssets.browse(PAGE_SIZE_LIMIT, null))
        .thenReturn(new FluentContinuation<>(assetContinuation, asset -> aFluentAsset()));

    List<FluentAsset> assets = range(0, NUMBER_OF_ASSETS).mapToObj(i -> aFluentAsset()).toList();
    when(assetContinuation.iterator()).thenReturn(assets.iterator());
  }

  private FluentAsset aFluentAsset() {
    return new FluentAssetImpl(contentFacetSupport, anAsset());
  }

  private AssetData anAsset() {
    AssetData asset = new AssetData();
    asset.setAssetId(ASSET_ID);
    asset.setPath(ASSET_PATH);
    return asset;
  }
}
