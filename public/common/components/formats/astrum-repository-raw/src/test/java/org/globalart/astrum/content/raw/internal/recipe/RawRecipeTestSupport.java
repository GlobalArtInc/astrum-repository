
package org.globalart.astrum.content.raw.internal.recipe;

import jakarta.inject.Provider;

import org.globalart.astrum.content.raw.RawContentFacet;
import org.globalart.astrum.repository.content.browse.BrowseFacet;
import org.globalart.astrum.repository.content.maintenance.SingleAssetMaintenanceFacet;
import org.globalart.astrum.repository.content.search.SearchFacet;
import org.globalart.astrum.repository.http.PartialFetchHandler;
import org.globalart.astrum.repository.raw.ContentDispositionHandler;
import org.globalart.astrum.repository.raw.internal.RawIndexHtmlForwardHandler;
import org.globalart.astrum.repository.raw.internal.RawSecurityFacet;
import org.globalart.astrum.repository.security.SecurityHandler;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.handlers.ConditionalRequestHandler;
import org.globalart.astrum.repository.view.handlers.ContentHeadersHandler;
import org.globalart.astrum.repository.view.handlers.ExceptionHandler;
import org.globalart.astrum.repository.view.handlers.HandlerContributor;
import org.globalart.astrum.repository.view.handlers.LastDownloadedHandler;
import org.globalart.astrum.repository.view.handlers.TimingHandler;

import org.junit.Rule;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnit;
import org.mockito.junit.MockitoRule;

public abstract class RawRecipeTestSupport
{
  @Rule
  public MockitoRule mockitoRule = MockitoJUnit.rule().silent();

  @Mock
  private ExceptionHandler exceptionHandler;

  @Mock
  private TimingHandler timingHandler;

  @Mock
  private RawIndexHtmlForwardHandler indexHtmlForwardHandler;

  @Mock
  private SecurityHandler securityHandler;

  @Mock
  private PartialFetchHandler partialFetchHandler;

  @Mock
  private RawContentHandler contentHandler;

  @Mock
  private ConditionalRequestHandler conditionalRequestHandler;

  @Mock
  private ContentHeadersHandler contentHeadersHandler;

  @Mock
  private LastDownloadedHandler lastDownloadedHandler;

  @Mock
  private HandlerContributor handlerContributor;

  @Mock
  private ContentDispositionHandler contentDispositionHandler;

  @Mock
  protected RawSecurityFacet securityFacet;

  private final Provider<RawSecurityFacet> securityFacetProvider = () -> securityFacet;

  @Mock
  protected ConfigurableViewFacet viewFacet;

  private final Provider<ConfigurableViewFacet> viewFacetProvider = () -> viewFacet;

  @Mock
  protected RawContentFacet contentFacet;

  private final Provider<RawContentFacet> contentFacetProvider = () -> contentFacet;

  @Mock
  protected SingleAssetMaintenanceFacet maintenanceFacet;

  private final Provider<SingleAssetMaintenanceFacet> maintenanceFacetProvider = () -> maintenanceFacet;

  @Mock
  protected SearchFacet searchFacet;

  private final Provider<SearchFacet> searchFacetProvider = () -> searchFacet;

  @Mock
  protected BrowseFacet browseFacet;

  private final Provider<BrowseFacet> browseFacetProvider = () -> browseFacet;

  protected <T extends RawRecipeSupport> void mockDependencies(final T underTest) {
    underTest.setDependencies(securityFacetProvider, viewFacetProvider, contentFacetProvider, maintenanceFacetProvider,
        searchFacetProvider, browseFacetProvider, exceptionHandler, timingHandler, indexHtmlForwardHandler,
        securityHandler, partialFetchHandler, contentHandler, conditionalRequestHandler, contentHeadersHandler,
        lastDownloadedHandler, handlerContributor, contentDispositionHandler);
  }
}
