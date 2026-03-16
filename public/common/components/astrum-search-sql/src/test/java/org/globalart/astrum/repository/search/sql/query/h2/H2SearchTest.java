
package org.globalart.astrum.repository.search.sql.query.h2;

import java.util.List;
import java.util.Optional;
import java.util.Set;
import java.util.UUID;
import java.util.function.Consumer;

import org.globalart.astrum.common.entity.DetachedEntityId;
import org.globalart.astrum.datastore.mybatis.handlers.ExternalMetadataTypeHandler;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.ConfigurationDAO;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.fluent.internal.FluentComponentImpl;
import org.globalart.astrum.repository.content.security.AssetPermissionChecker;
import org.globalart.astrum.repository.content.store.BlobRefTypeHandler;
import org.globalart.astrum.repository.content.store.ComponentData;
import org.globalart.astrum.repository.content.store.ContentRepositoryData;
import org.globalart.astrum.repository.content.store.InternalIds;
import org.globalart.astrum.repository.content.store.example.TestAssetBlobDAO;
import org.globalart.astrum.repository.content.store.example.TestAssetDAO;
import org.globalart.astrum.repository.content.store.example.TestComponentDAO;
import org.globalart.astrum.repository.content.store.example.TestContentRepositoryDAO;
import org.globalart.astrum.repository.rest.SearchMappings;
import org.globalart.astrum.repository.rest.internal.DefaultSearchMappings;
import org.globalart.astrum.repository.search.SearchRequest;
import org.globalart.astrum.repository.search.normalize.VersionNormalizerService;
import org.globalart.astrum.repository.search.sql.SearchMappingService;
import org.globalart.astrum.repository.search.sql.SqlSearchQueryContribution;
import org.globalart.astrum.repository.search.sql.index.SearchRecordProducer;
import org.globalart.astrum.repository.search.sql.query.DatabaseTypeDetector;
import org.globalart.astrum.repository.search.sql.query.DefaultSqlSearchQueryContribution;
import org.globalart.astrum.repository.search.sql.query.ExpressionBuilder;
import org.globalart.astrum.repository.search.sql.query.KeywordSqlSearchQueryContribution;
import org.globalart.astrum.repository.search.sql.query.SearchRequestModifier;
import org.globalart.astrum.repository.search.sql.query.SqlSearchService;
import org.globalart.astrum.repository.search.sql.query.SqlSearchSortUtil;
import org.globalart.astrum.repository.search.sql.query.security.SqlSearchPermissionBuilder;
import org.globalart.astrum.repository.search.sql.store.SearchRecordData;
import org.globalart.astrum.repository.search.sql.store.SearchStore;
import org.globalart.astrum.repository.search.sql.store.SearchTableDAO;
import org.globalart.astrum.testcommon.extensions.LoggingExtension;
import org.globalart.astrum.testcommon.extensions.LoggingExtension.CaptureLogsFor;
import org.globalart.astrum.testcommon.extensions.LoggingExtension.TestLogAccessor;
import org.globalart.astrum.testdb.DataSessionConfiguration;
import org.globalart.astrum.testdb.DatabaseExtension;
import org.globalart.astrum.testdb.DatabaseTest;
import org.globalart.astrum.testdb.TestDataSessionSupplier;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.slf4j.event.Level;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.empty;
import static org.hamcrest.Matchers.hasSize;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.lenient;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

/*
 * This class contains tests of common search patterns
 */
@ExtendWith(MockitoExtension.class)
@ExtendWith({DatabaseExtension.class, LoggingExtension.class})
class H2SearchTest
{
  private static final String MAVEN_SLF4J_PROVIDER = "maven-slf4j-provider";

  private static final String ORG_APACHE_MAVEN = "org.apache.maven";

  @CaptureLogsFor(value = SearchTableDAO.class, level = Level.TRACE)
  TestLogAccessor accessor;

  @DataSessionConfiguration(
      daos = {SearchTableDAO.class, TestContentRepositoryDAO.class, TestComponentDAO.class, TestAssetBlobDAO.class,
          TestAssetDAO.class, ConfigurationDAO.class},
      typeHandlers = {BlobRefTypeHandler.class, ExternalMetadataTypeHandler.class}, h2 = true, postgresql = false)
  protected TestDataSessionSupplier sessionRule;

  @Mock
  SqlSearchPermissionBuilder permissionBuilder;

  @Mock
  AssetPermissionChecker permissionChecker;

  private int repositoryId;

  @BeforeEach
  void setup() {
    ContentRepositoryData repository = new ContentRepositoryData();
    repository.setConfigRepositoryId(new DetachedEntityId(UUID.randomUUID().toString()));
    sessionRule.callDAO(TestContentRepositoryDAO.class, dao -> dao.createContentRepository(repository));
    repositoryId = InternalIds.contentRepositoryId(repository);

    when(permissionBuilder.build(any())).thenReturn(Optional.empty());
    lenient().when(permissionChecker.findPermittedAssets(any(), any(), any())).thenAnswer(i -> i.getArgument(0));
  }

  @DatabaseTest(h2 = true)
  void testExact() {
    createComponent(c -> {
      c.setNamespace(ORG_APACHE_MAVEN);
      c.setName(MAVEN_SLF4J_PROVIDER);
      c.setVersion("4.0.0");
    });

    // Exact
    assertThat(underTest().search(nameRequest(MAVEN_SLF4J_PROVIDER)).getSearchResults(), hasSize(1));
    assertThat(underTest().search(namespaceRequest(ORG_APACHE_MAVEN)).getSearchResults(), hasSize(1));

    // maven colon-separator
    assertThat(underTest().search(keywordRequest("org.apache.maven:maven-slf4j-provider")).getSearchResults(),
        hasSize(1));
    assertThat(underTest().search(keywordRequest("org.apache.maven:maven-slf4j-provider:4.0.0")).getSearchResults(),
        hasSize(1));
  }

  @DatabaseTest(h2 = true)
  void testTokenization() {
    createComponent(c -> {
      c.setNamespace(ORG_APACHE_MAVEN);
      c.setName(MAVEN_SLF4J_PROVIDER);
    });

    // spaces
    assertThat(underTest().search(keywordRequest("maven apache")).getSearchResults(), hasSize(1));
    assertThat(underTest().search(keywordRequest("maven provider")).getSearchResults(), hasSize(1));
    assertThat(underTest().search(keywordRequest(ORG_APACHE_MAVEN)).getSearchResults(), hasSize(1));
    assertThat(underTest().search(keywordRequest(MAVEN_SLF4J_PROVIDER)).getSearchResults(), hasSize(1));
    assertThat(underTest().search(keywordRequest("maven*provider apache")).getSearchResults(), hasSize(1));
  }

  @DatabaseTest(h2 = true)
  void testStartsWith() {
    createComponent(c -> {
      c.setNamespace(ORG_APACHE_MAVEN);
      c.setName(MAVEN_SLF4J_PROVIDER);
    });

    assertThat(underTest().search(nameRequest("maven*provider")).getSearchResults(), hasSize(1));
    assertThat(underTest().search(nameRequest("maven*slf4j*provider")).getSearchResults(), hasSize(1));
    assertThat(underTest().search(nameRequest("maven-slf4*")).getSearchResults(), hasSize(1));
    assertThat(underTest().search(namespaceRequest("org.apache*")).getSearchResults(), hasSize(1));
    assertThat(underTest().search(nameRequest("mav*-slf4j-provider")).getSearchResults(), hasSize(1));

    assertThat(underTest().search(nameRequest("slf4j-provider")).getSearchResults(), empty());
    assertThat(underTest().search(nameRequest("slf4j-provider*")).getSearchResults(), empty());
    assertThat(underTest().search(nameRequest("slf4j*")).getSearchResults(), empty());
  }

  private SqlSearchService underTest() {
    return new SqlSearchService(new SearchStore(sessionRule, 100, 100),
        new SqlSearchSortUtil(searchDatabase(), searchMappings()), List.of(), expressionBuilder(),
        new SearchRequestModifier(searchQueryContributions()), conditionFactory(), Set.of(), mock(), permissionChecker);
  }

  private void createComponent(final Consumer<ComponentData> mutator) {
    ComponentData component = new ComponentData();
    component.setName(UUID.randomUUID().toString());
    component.setNamespace("");
    component.setVersion("1");
    component.setRepositoryId(repositoryId);
    component.setKind("");

    mutator.accept(component);

    sessionRule.callDAO(TestComponentDAO.class, dao -> dao.createComponent(component, true));

    // Fake asset
    FluentAsset asset = mock();
    when(asset.blob()).thenReturn(Optional.empty());
    when(asset.path()).thenReturn("/foo");

    FluentComponent result = sessionRule
        .withDAO(TestComponentDAO.class, dao -> dao.readComponent(InternalIds.internalComponentId(component)))
        .map(c -> new FluentComponentImpl(mock(), c, List.of(asset)))
        .orElseThrow();

    Repository repository = mock();
    when(repository.getName()).thenReturn("fake");
    Format format = mock();
    when(format.getValue()).thenReturn("test");
    when(repository.getFormat()).thenReturn(format);
    ContentFacet facet = mock();
    when(repository.facet(ContentFacet.class)).thenReturn(facet);
    when(facet.contentRepositoryId()).thenReturn(repositoryId);

    SearchRecordData record = recordProducer().createSearchRecord(result, repository).orElseThrow();
    store().save(record);
  }

  private static SearchRecordProducer recordProducer() {
    DatabaseTypeDetector typeDetector = mock();
    lenient().when(typeDetector.isH2()).thenReturn(true);
    when(typeDetector.isPostgreSQL()).thenReturn(false);
    return new SearchRecordProducer(List.of(), new VersionNormalizerService(List.of()), List.of(), List.of(), Set.of(),
        typeDetector);
  }

  private SearchStore store() {
    return new SearchStore(sessionRule, 100, 100);
  }

  private ExpressionBuilder expressionBuilder() {
    return new ExpressionBuilder(permissionBuilder,
        searchQueryContributions(),
        searchMappings());
  }

  private static SearchRequest keywordRequest(final String keyword) {
    return SearchRequest.builder()
        .searchFilter("keyword", keyword)
        .build();
  }

  private static SearchRequest nameRequest(final String name) {
    return SearchRequest.builder()
        .searchFilter("name", name)
        .build();
  }

  private static SearchRequest namespaceRequest(final String namespace) {
    return SearchRequest.builder()
        .searchFilter("namespace", namespace)
        .build();
  }

  private static List<SqlSearchQueryContribution> searchQueryContributions() {
    SearchMappingService mapping = new SearchMappingService(searchMappings());
    DefaultSqlSearchQueryContribution def = new DefaultSqlSearchQueryContribution();
    def.init(mapping, false);

    KeywordSqlSearchQueryContribution keyword = new KeywordSqlSearchQueryContribution();
    keyword.init(mapping, false);

    return List.of(def, keyword);
  }

  private static List<SearchMappings> searchMappings() {
    return List.of(new DefaultSearchMappings());
  }

  private static H2SearchConditionFactory conditionFactory() {
    return new H2SearchConditionFactory(searchDatabase());
  }

  private static H2SearchDB searchDatabase() {
    return new H2SearchDB();
  }
}
