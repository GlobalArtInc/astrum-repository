
package org.globalart.astrum.content.maven.internal.recipe;

import jakarta.inject.Provider;

import org.globalart.astrum.content.maven.MavenArchetypeCatalogFacet;
import org.globalart.astrum.content.maven.MavenContentFacet;
import org.globalart.astrum.repository.content.browse.BrowseFacet;
import org.globalart.astrum.repository.content.search.SearchFacet;
import org.globalart.astrum.repository.http.PartialFetchHandler;
import org.globalart.astrum.repository.maven.ContentDispositionHandler;
import org.globalart.astrum.repository.maven.MavenMetadataRebuildFacet;
import org.globalart.astrum.repository.maven.MavenPathParser;
import org.globalart.astrum.repository.maven.RemoveSnapshotsFacet;
import org.globalart.astrum.repository.maven.internal.MavenSecurityFacet;
import org.globalart.astrum.repository.maven.internal.VersionPolicyHandler;
import org.globalart.astrum.repository.routing.RoutingRuleHandler;
import org.globalart.astrum.repository.security.SecurityHandler;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Matcher;
import org.globalart.astrum.repository.view.Route;
import org.globalart.astrum.repository.view.handlers.BrowseUnsupportedHandler;
import org.globalart.astrum.repository.view.handlers.ConditionalRequestHandler;
import org.globalart.astrum.repository.view.handlers.ContentHeadersHandler;
import org.globalart.astrum.repository.view.handlers.ExceptionHandler;
import org.globalart.astrum.repository.view.handlers.HandlerContributor;
import org.globalart.astrum.repository.view.handlers.LastDownloadedHandler;
import org.globalart.astrum.repository.view.handlers.TimingHandler;

import com.google.common.collect.ImmutableList;
import org.junit.Rule;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnit;
import org.mockito.junit.MockitoRule;

import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

public abstract class MavenRecipeTestSupport
{
  @Rule
  public MockitoRule mockitoRule = MockitoJUnit.rule().silent();

  @Mock
  private SecurityHandler securityHandler;

  @Mock
  private VersionPolicyHandler versionPolicyHandler;

  @Mock
  private LastDownloadedHandler lastDownloadedHandler;

  @Mock
  private BrowseUnsupportedHandler browseUnsupportedHandler;

  @Mock
  private ContentHeadersHandler contentHeadersHandler;

  @Mock
  private PartialFetchHandler partialFetchHandler;

  @Mock
  private TimingHandler timingHandler;

  @Mock
  private MavenPathParser mavenPathParser;

  @Mock
  private RoutingRuleHandler routingRuleHandler;

  @Mock
  private ExceptionHandler exceptionHandler;

  @Mock
  private ConditionalRequestHandler conditionalRequestHandler;

  @Mock
  private MavenMetadataRebuildHandler mavenMetadataRebuildHandler;

  @Mock
  private MavenContentHandler mavenContentHandler;

  @Mock
  private MavenArchetypeCatalogHandler archetypeCatalogHandler;

  @Mock
  private ContentDispositionHandler contentDispositionHandler;

  @Mock
  private HandlerContributor handlerContributor;

  @Mock
  protected ConfigurableViewFacet viewFacet;

  private final Provider<ConfigurableViewFacet> viewFacetProvider = () -> viewFacet;

  @Mock
  protected BrowseFacet browseFacet;

  private final Provider<BrowseFacet> browseFacetProvider = () -> browseFacet;

  @Mock
  protected MavenSecurityFacet securityFacet;

  private final Provider<MavenSecurityFacet> securityFacetProvider = () -> securityFacet;

  @Mock
  protected MavenContentFacet mavenContentFacet;

  private final Provider<MavenContentFacet> mavenContentFacetProvider = () -> mavenContentFacet;

  @Mock
  protected MavenMaintenanceFacet mavenMaintenanceFacet;

  private final Provider<MavenMaintenanceFacet> mavenMaintenanceFacetProvider = () -> mavenMaintenanceFacet;

  @Mock
  protected SearchFacet searchFacet;

  private final Provider<SearchFacet> searchFacetProvider = () -> searchFacet;

  @Mock
  protected RemoveSnapshotsFacet removeSnapshotsFacet;

  private final Provider<RemoveSnapshotsFacet> removeSnapshotsFacetProvider = () -> removeSnapshotsFacet;

  @Mock
  protected MavenMetadataRebuildFacet mavenMetadataRebuildFacet;

  private final Provider<MavenMetadataRebuildFacet> mavenMetadataRebuildFacetProvider = () -> mavenMetadataRebuildFacet;

  @Mock
  protected MavenArchetypeCatalogFacet mavenArchetypeCatalogFacet;

  private final Provider<MavenArchetypeCatalogFacet> mavenArchetypeCatalogFacetProvider =
      () -> mavenArchetypeCatalogFacet;

  protected <T extends MavenRecipeSupport> void mockHandlers(T underTest) {
    underTest.setBrowseUnsupportedHandler(browseUnsupportedHandler);
    when(browseUnsupportedHandler.getRoute()).thenReturn(
        new Route(mock(Matcher.class), ImmutableList.of(securityHandler, browseUnsupportedHandler)));
    underTest.setContentHeadersHandler(contentHeadersHandler);
    underTest.setPartialFetchHandler(partialFetchHandler);
    underTest.setTimingHandler(timingHandler);
    underTest.setMavenPathParser(mavenPathParser);
    underTest.setSecurityHandler(securityHandler);
    underTest.setRoutingHandler(routingRuleHandler);
    underTest.setExceptionHandler(exceptionHandler);
    underTest.setConditionalRequestHandler(conditionalRequestHandler);
    underTest.setHandlerContributor(handlerContributor);
    underTest.setContentDispositionHandler(contentDispositionHandler);
    underTest.setVersionPolicyHandler(versionPolicyHandler);
    underTest.setLastDownloadedHandler(lastDownloadedHandler);
    underTest.setMavenMetadataRebuildHandler(mavenMetadataRebuildHandler);
    underTest.setArchetypeCatalogHandler(archetypeCatalogHandler);
    underTest.setMavenContentHandler(mavenContentHandler);
  }

  protected <T extends MavenRecipeSupport> void mockFacets(T underTest) {
    underTest.setSecurityFacet(securityFacetProvider);
    underTest.setMavenContentFacet(mavenContentFacetProvider);
    underTest.setBrowseFacet(browseFacetProvider);
    underTest.setViewFacet(viewFacetProvider);
    underTest.setMavenMaintenanceFacet(mavenMaintenanceFacetProvider);
    underTest.setSearchFacet(searchFacetProvider);
    underTest.setRemoveSnapshotsFacet(removeSnapshotsFacetProvider);
    underTest.setMavenMetadataRebuildFacet(mavenMetadataRebuildFacetProvider);
    underTest.setMavenArchetypeCatalogFacet(mavenArchetypeCatalogFacetProvider);
  }
}
