
package org.globalart.astrum.cleanup.internal.content.service;

import java.time.Duration;
import java.time.OffsetDateTime;
import java.util.Collection;
import java.util.Collections;
import java.util.Date;
import java.util.List;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import java.util.stream.Stream;

import jakarta.ws.rs.WebApplicationException;

import org.globalart.astrum.cleanup.content.search.CleanupBrowseServiceFactory;
import org.globalart.astrum.cleanup.content.search.CleanupComponentBrowse;
import org.globalart.astrum.cleanup.content.search.DefaultCleanupComponentBrowse;
import org.globalart.astrum.cleanup.internal.datastore.search.criteria.LastBlobUpdatedCleanupEvaluator;
import org.globalart.astrum.cleanup.internal.datastore.search.criteria.RegexCleanupEvaluator;
import org.globalart.astrum.cleanup.preview.CleanupPreviewSummaryXO;
import org.globalart.astrum.cleanup.internal.storage.CleanupPolicyData;
import org.globalart.astrum.cleanup.storage.CleanupPolicy;
import org.globalart.astrum.cleanup.storage.CleanupPolicyCriteria;
import org.globalart.astrum.cleanup.storage.CleanupPolicyPreviewXO;
import org.globalart.astrum.cleanup.storage.CleanupPolicyReleaseType;
import org.globalart.astrum.cleanup.storage.CleanupPolicyStorage;
import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.extdirect.model.PagedResponse;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.facet.ContentFacetStores;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.fluent.FluentComponents;
import org.globalart.astrum.repository.content.fluent.internal.FluentComponentImpl;
import org.globalart.astrum.repository.content.store.AssetBlobData;
import org.globalart.astrum.repository.content.store.AssetData;
import org.globalart.astrum.repository.content.store.AssetStore;
import org.globalart.astrum.repository.content.store.ComponentData;
import org.globalart.astrum.repository.query.QueryOptions;
import org.globalart.astrum.repository.rest.api.AssetXO;
import org.globalart.astrum.repository.rest.api.ComponentXO;
import org.globalart.astrum.rest.ValidationErrorsException;

import com.google.common.collect.ForwardingCollection;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.mockito.ArgumentCaptor;
import org.mockito.Captor;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.allOf;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.hasEntry;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyInt;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.*;

import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.*;

@ExtendWith(MockitoExtension.class)
class CleanupPreviewHelperImplTest

{

  private static final OffsetDateTime NOW = OffsetDateTime.now();

  private static final Duration SUMMARY_TIMEOUT = Duration.ofSeconds(30);

  @Mock
  private CleanupPolicyStorage cleanupPolicyStorage;

  @Mock
  private Duration previewTimeout;

  @Mock
  private CleanupBrowseServiceFactory browseServiceFactory;

  @Mock
  private CleanupComponentBrowse cleanupComponentBrowse;

  private CleanupPreviewHelperImpl underTest;

  @Captor
  private ArgumentCaptor<CleanupPolicy> cleanPolicyCaptor;

  @BeforeEach
  void setup() {
    lenient().when(cleanupPolicyStorage.newCleanupPolicy()).thenAnswer(invocation -> new CleanupPolicyData());
    lenient().when(browseServiceFactory.getPreviewService()).thenReturn(cleanupComponentBrowse);
    underTest = new CleanupPreviewHelperImpl(cleanupPolicyStorage, previewTimeout, SUMMARY_TIMEOUT,
        browseServiceFactory, false);
  }

  @Test
  void testGetSearchResults() {
    when(previewTimeout.toMillis()).thenReturn(60L);

    Collection<Component> components = getComponentCollection();
    when(cleanupComponentBrowse.browseByPage(any(CleanupPolicy.class), any(Repository.class), any(QueryOptions.class)))
        .thenReturn(new PagedResponse<>(2, components));

    CleanupPolicyPreviewXO cleanupPolicyPreviewXO = createCleanupPolicyPreviewXO();
    Repository repository = createRepository();
    QueryOptions queryOptions = createQueryOptions();

    PagedResponse<ComponentXO> componentPagedResponse =
        underTest.getSearchResults(cleanupPolicyPreviewXO, repository, queryOptions);

    verify(previewTimeout).toMillis();
    verify(cleanupPolicyStorage).newCleanupPolicy();
    verify(browseServiceFactory).getPreviewService();
    verify(cleanupComponentBrowse).browseByPage(cleanPolicyCaptor.capture(), eq(repository), eq(queryOptions));

    assertCleanupPolicy(cleanPolicyCaptor.getValue());

    assertThat(componentPagedResponse.getTotal(), is(2L));
    assertComponentResultXO(componentPagedResponse.getData().stream(), false);
  }

  @Test
  void testGetSearchResultsStream() {
    Collection<Component> componentCollection = getComponentCollection();
    ContentFacetSupport contentFacetSupport = mock(ContentFacetSupport.class);
    ContentFacetStores contentFacetStores = mock(ContentFacetStores.class);
    AssetStore assetStore = mock(AssetStore.class);

    when(contentFacetSupport.stores()).thenReturn(contentFacetStores);
    when(contentFacetStores.assetStore()).thenReturn(assetStore);

    AssetBlobData assetBlob = new AssetBlobData();
    assetBlob.setAssetBlobId(1);
    assetBlob.setBlobCreated(NOW);

    AssetData asset = new AssetData();
    asset.setPath("/assetPath");
    asset.setBlobStoreName("blobStoreName");
    asset.setAssetBlobSize(100L);
    asset.setLastDownloaded(NOW);
    asset.setCreated(NOW);
    asset.setAssetBlob(assetBlob);

    when(assetStore.browseComponentAssets(any(Component.class))).thenReturn(Collections.singleton(asset));

    Stream<FluentComponent> fluentComponentStream = componentCollection.stream()
        .map(component -> new FluentComponentImpl(contentFacetSupport, component));

    when(cleanupComponentBrowse.browseIncludingAssets(any(CleanupPolicy.class), any(Repository.class)))
        .thenReturn(fluentComponentStream);

    CleanupPolicyPreviewXO cleanupPolicyPreviewXO = createCleanupPolicyPreviewXO();
    Repository repository = createRepository();

    Stream<ComponentXO> componentXOStream =
        underTest.getSearchResultsStream(cleanupPolicyPreviewXO, repository, createQueryOptions());

    verify(cleanupPolicyStorage).newCleanupPolicy();
    verify(browseServiceFactory).getPreviewService();
    verify(cleanupComponentBrowse).browseIncludingAssets(cleanPolicyCaptor.capture(), eq(repository));

    assertCleanupPolicy(cleanPolicyCaptor.getValue());
    assertComponentResultXO(componentXOStream, true);
  }

  @Test
  void testGetSearchResultsStream_withNullBlob() {
    Collection<Component> componentCollection = getComponentCollection();
    ContentFacetSupport contentFacetSupport = mock(ContentFacetSupport.class);
    ContentFacetStores contentFacetStores = mock(ContentFacetStores.class);
    AssetStore assetStore = mock(AssetStore.class);

    when(contentFacetSupport.stores()).thenReturn(contentFacetStores);
    when(contentFacetStores.assetStore()).thenReturn(assetStore);

    AssetData asset = new AssetData();
    asset.setPath("/assetPath");
    asset.setBlobStoreName("blobStoreName");
    asset.setAssetBlobSize(100L);
    asset.setLastDownloaded(NOW);
    asset.setCreated(NOW);
    asset.setAssetBlob(null);

    when(assetStore.browseComponentAssets(any(Component.class))).thenReturn(Collections.singleton(asset));

    Stream<FluentComponent> fluentComponentStream = componentCollection.stream()
        .map(component -> new FluentComponentImpl(contentFacetSupport, component));

    when(cleanupComponentBrowse.browseIncludingAssets(any(CleanupPolicy.class), any(Repository.class)))
        .thenReturn(fluentComponentStream);

    CleanupPolicyPreviewXO cleanupPolicyPreviewXO = createCleanupPolicyPreviewXO();
    Repository repository = createRepository();

    Stream<ComponentXO> componentXOStream =
        underTest.getSearchResultsStream(cleanupPolicyPreviewXO, repository, createQueryOptions());

    verify(cleanupPolicyStorage).newCleanupPolicy();
    verify(browseServiceFactory).getPreviewService();
    verify(cleanupComponentBrowse).browseIncludingAssets(cleanPolicyCaptor.capture(), eq(repository));

    assertCleanupPolicy(cleanPolicyCaptor.getValue());

    List<ComponentXO> componentsResult = componentXOStream.toList();
    assertThat(componentsResult, hasSize(2));

    componentsResult.forEach(componentXO -> {
      List<AssetXO> assets = componentXO.getAssets();
      assertThat(assets, hasSize(1));

      AssetXO assetXO = assets.get(0);
      assertThat(assetXO.getPath(), is("/assetPath"));
      assertThat(assetXO.getBlobStoreName(), is("blobStoreName"));
      assertThat(assetXO.getFileSize(), is(100L));

      Date nowDate = Date.from(NOW.toInstant());
      assertThat(assetXO.getLastDownloaded(), is(nowDate));
      assertThat(assetXO.getBlobCreated(), is((Date) null));
    });
  }

  @Test
  void testGetSearchResultsReportsUnsupportedCriteriaAsValidationError() {
    when(previewTimeout.toMillis()).thenReturn(10_000L);
    when(cleanupComponentBrowse.browseByPage(any(CleanupPolicy.class), any(Repository.class), any(QueryOptions.class)))
        .thenThrow(new UnsupportedOperationException("Criteria of type(s) [retain] not supported"));

    ValidationErrorsException e = assertThrows(ValidationErrorsException.class,
        () -> underTest.getSearchResults(createCleanupPolicyPreviewXO(), createRepository(), createQueryOptions()));

    assertThat(e.getValidationErrors().get(0).getMessage(), containsString("[retain] not supported"));
  }

  @Test
  void testGetSearchResultsReportsTimeout() throws Exception {
    CountDownLatch release = new CountDownLatch(1);
    when(previewTimeout.toMillis()).thenReturn(100L);
    when(cleanupComponentBrowse.browseByPage(any(CleanupPolicy.class), any(Repository.class), any(QueryOptions.class)))
        .thenAnswer(invocation -> {
          release.await(10, TimeUnit.SECONDS);
          return new PagedResponse<>(-1, List.of());
        });

    try {
      WebApplicationException e = assertThrows(WebApplicationException.class,
          () -> underTest.getSearchResults(createCleanupPolicyPreviewXO(), createRepository(), createQueryOptions()));

      assertThat(e.getResponse().getStatus(), is(500));
      assertThat(e.getResponse().getEntity(), is("A timeout occurred while computing the preview results."));
    }
    finally {
      release.countDown();
    }
  }

  @Test
  void testGetSummaryCountsMatchingComponentsWithAllTheirAssets() {
    Repository repository = createRepository();
    FluentComponent first = fluentComponent("app", "1.0", asset("/app/1.0/app-1.0.zip", 100L),
        asset("/app/1.0/app-1.0.txt", 250L));
    FluentComponent second = fluentComponent("lib", "2.0", asset("/lib/2.0/lib-2.0.zip", 1_000L));
    FluentComponent unmatched = fluentComponent("doc", "3.0", asset("/doc/3.0/doc-3.0.txt", 5_000L));
    underTest = withRealBrowse(repository, first, second, unmatched);

    CleanupPolicyPreviewXO previewXO = previewXO(new CleanupPolicyCriteria(null, null, null, ".*\\.zip", null, null));
    CleanupPreviewSummaryXO summary = underTest.getSummary(previewXO, repository, null);

    assertThat(summary.getRepository(), is("repositoryName"));
    assertThat(summary.getComponentCount(), is(2L));
    assertThat(summary.getAssetCount(), is(3L));
    assertThat(summary.getTotalSize(), is(1_350L));
    assertThat(summary.isComplete(), is(true));
  }

  @Test
  void testGetSummaryAppliesTextFilter() {
    Repository repository = createRepository();
    FluentComponent first = fluentComponent("app", "1.0", asset("/app/1.0/app-1.0.zip", 100L));
    FluentComponent second = fluentComponent("lib", "2.0", asset("/lib/2.0/lib-2.0.zip", 1_000L));
    underTest = withRealBrowse(repository, first, second);

    CleanupPolicyPreviewXO previewXO = previewXO(new CleanupPolicyCriteria(null, null, null, ".*\\.zip", null, null));
    CleanupPreviewSummaryXO summary = underTest.getSummary(previewXO, repository, "li");

    assertThat(summary.getComponentCount(), is(1L));
    assertThat(summary.getAssetCount(), is(1L));
    assertThat(summary.getTotalSize(), is(1_000L));
    assertThat(summary.isComplete(), is(true));
  }

  @Test
  void testGetSummaryReportsUnsupportedCriteriaAsValidationError() {
    Repository repository = createRepository();
    underTest = withRealBrowse(repository);

    CleanupPolicyPreviewXO previewXO = previewXO(new CleanupPolicyCriteria(30, null, null, null, 3, "version"));

    ValidationErrorsException e =
        assertThrows(ValidationErrorsException.class, () -> underTest.getSummary(previewXO, repository, null));

    assertThat(e.getValidationErrors().get(0).getMessage(), containsString("The preview cannot evaluate this policy"));
    assertThat(e.getValidationErrors().get(0).getMessage(), containsString("retain"));
  }

  @Test
  void testGetSummaryWithoutCriteriaDeletesNothing() {
    Repository repository = createRepository();

    CleanupPreviewSummaryXO summary = underTest.getSummary(
        previewXO(new CleanupPolicyCriteria(null, null, null, null, null, null)), repository, null);

    assertThat(summary.getComponentCount(), is(0L));
    assertThat(summary.getAssetCount(), is(0L));
    assertThat(summary.getTotalSize(), is(0L));
    assertThat(summary.isComplete(), is(true));
    verify(cleanupComponentBrowse, never()).browseIncludingAssets(any(), any());
  }

  @Test
  void testGetSummaryReturnsPartialTotalsWhenTimeLimitIsReached() {
    Repository repository = createRepository();
    FluentComponent first = fluentComponent("app", "1.0", asset("/app-1.0.zip", 100L));
    FluentComponent second = fluentComponent("lib", "2.0", asset("/lib-2.0.zip", 200L), asset("/lib-2.0.pom", 5L));
    FluentComponent never = mock(FluentComponent.class);
    CountDownLatch release = new CountDownLatch(1);

    when(cleanupComponentBrowse.browseIncludingAssets(any(CleanupPolicy.class), eq(repository)))
        .thenReturn(Stream.concat(Stream.of(first, second), Stream.generate(() -> {
          try {
            release.await(10, TimeUnit.SECONDS);
          }
          catch (InterruptedException e) {
            Thread.currentThread().interrupt();
          }
          return never;
        }).limit(1)));
    underTest = new CleanupPreviewHelperImpl(cleanupPolicyStorage, previewTimeout, Duration.ofMillis(200),
        browseServiceFactory, false);

    try {
      CleanupPreviewSummaryXO summary = underTest.getSummary(
          previewXO(new CleanupPolicyCriteria(null, null, null, ".*\\.zip", null, null)), repository, null);

      assertThat(summary.getComponentCount(), is(2L));
      assertThat(summary.getAssetCount(), is(3L));
      assertThat(summary.getTotalSize(), is(305L));
      assertThat(summary.isComplete(), is(false));
    }
    finally {
      release.countDown();
    }
    verify(never, after(500).never()).assets();
  }

  private CleanupPreviewHelperImpl withRealBrowse(final Repository repository, final FluentComponent... components) {
    DefaultCleanupComponentBrowse browse = new DefaultCleanupComponentBrowse(
        List.of(new LastBlobUpdatedCleanupEvaluator()), List.of(new RegexCleanupEvaluator()));
    ContentFacet contentFacet = mock(ContentFacet.class);
    FluentComponents fluentComponents = mock(FluentComponents.class);
    lenient().when(repository.facet(ContentFacet.class)).thenReturn(contentFacet);
    lenient().when(contentFacet.components()).thenReturn(fluentComponents);
    lenient().when(fluentComponents.browseEager(anyInt(), any()))
        .thenReturn(new ListContinuation<>(List.of(components)));
    CleanupBrowseServiceFactory factory = mock(CleanupBrowseServiceFactory.class);
    when(factory.getPreviewService()).thenReturn(browse);
    return new CleanupPreviewHelperImpl(cleanupPolicyStorage, previewTimeout, SUMMARY_TIMEOUT, factory, false);
  }

  private static FluentComponent fluentComponent(final String name, final String version, final FluentAsset... assets) {
    FluentComponent component = mock(FluentComponent.class);
    lenient().when(component.name()).thenReturn(name);
    lenient().when(component.namespace()).thenReturn("");
    lenient().when(component.version()).thenReturn(version);
    lenient().when(component.assets()).thenReturn(List.of(assets));
    return component;
  }

  private static FluentAsset asset(final String path, final long size) {
    FluentAsset asset = mock(FluentAsset.class);
    lenient().when(asset.path()).thenReturn(path);
    lenient().when(asset.assetBlobSize()).thenReturn(size);
    return asset;
  }

  private static CleanupPolicyPreviewXO previewXO(final CleanupPolicyCriteria criteria) {
    CleanupPolicyPreviewXO previewXO = new CleanupPolicyPreviewXO();
    previewXO.setRepositoryName("repositoryName");
    previewXO.setCriteria(criteria);
    return previewXO;
  }

  private static class ListContinuation<E>
      extends ForwardingCollection<E>
      implements Continuation<E>
  {
    private final Collection<E> items;

    ListContinuation(final Collection<E> items) {
      this.items = items;
    }

    @Override
    protected Collection<E> delegate() {
      return items;
    }

    @Override
    public String nextContinuationToken() {
      return null;
    }
  }

  private static void assertComponentResultXO(final Stream<ComponentXO> componentXOStream, final boolean withAssets) {
    List<ComponentXO> componentsResult = componentXOStream.toList();
    assertThat(componentsResult, hasSize(2));

    ComponentXO componentXO1 = componentsResult.get(0);
    assertThat(componentXO1.getRepository(), is("repositoryName"));
    assertThat(componentXO1.getGroup(), is("my-namespace-1"));
    assertThat(componentXO1.getName(), is("my-component-1"));
    assertThat(componentXO1.getVersion(), is("1.0"));
    assertThat(componentXO1.getFormat(), is("test-format"));

    ComponentXO componentXO2 = componentsResult.get(1);
    assertThat(componentXO2.getRepository(), is("repositoryName"));
    assertThat(componentXO2.getGroup(), is("my-namespace-2"));
    assertThat(componentXO2.getName(), is("my-component-2"));
    assertThat(componentXO2.getVersion(), is("2.0"));
    assertThat(componentXO2.getFormat(), is("test-format"));

    if (withAssets) {
      componentsResult.forEach(componentXO -> {
        List<AssetXO> assets = componentXO.getAssets();
        assertThat(assets, hasSize(1));

        AssetXO asset = assets.get(0);
        assertThat(asset.getPath(), is("/assetPath"));
        assertThat(asset.getBlobStoreName(), is("blobStoreName"));
        assertThat(asset.getFileSize(), is(100L));

        Date nowDate = Date.from(NOW.toInstant());
        assertThat(asset.getLastDownloaded(), is(nowDate));
        assertThat(asset.getBlobCreated(), is(nowDate));
      });
    }
  }

  private static void assertCleanupPolicy(final CleanupPolicy cleanupPolicy) {
    assertThat(cleanupPolicy.getCriteria(),
        allOf(hasEntry(LAST_BLOB_UPDATED_KEY, "86400"), hasEntry(LAST_DOWNLOADED_KEY, "86400"),
            hasEntry(IS_PRERELEASE_KEY, "false"), hasEntry(REGEX_KEY, "criteriaAssetRegex"),
            hasEntry(RETAIN_KEY, "1"), hasEntry(RETAIN_SORT_BY_KEY, "name")));
    assertThat(cleanupPolicy.getName(), is("preview"));
  }

  private static CleanupPolicyPreviewXO createCleanupPolicyPreviewXO() {
    CleanupPolicyPreviewXO cleanupPolicyPreviewXO = new CleanupPolicyPreviewXO();
    cleanupPolicyPreviewXO.setRepositoryName("repositoryName");
    CleanupPolicyCriteria criteria =
        new CleanupPolicyCriteria(1, 1, CleanupPolicyReleaseType.RELEASES, "criteriaAssetRegex", 1, "name");
    cleanupPolicyPreviewXO.setCriteria(criteria);
    return cleanupPolicyPreviewXO;
  }

  private static Collection<Component> getComponentCollection() {
    ComponentData component1 = new ComponentData();
    component1.setName("my-component-1");
    component1.setNamespace("my-namespace-1");
    component1.setVersion("1.0");
    ComponentData component2 = new ComponentData();
    component2.setName("my-component-2");
    component2.setNamespace("my-namespace-2");
    component2.setVersion("2.0");
    return List.of(component1, component2);
  }

  private static QueryOptions createQueryOptions() {
    return new QueryOptions("my-component-1", "name", "asc", 0, 100);
  }

  private static Repository createRepository() {
    Repository repository = mock(Repository.class);
    lenient().when(repository.getName()).thenReturn("repositoryName");
    Format formatMock = mock(Format.class);
    lenient().when(repository.getFormat()).thenReturn(formatMock);
    lenient().when(formatMock.getValue()).thenReturn("test-format");
    return repository;
  }
}
