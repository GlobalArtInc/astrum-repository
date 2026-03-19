
package org.globalart.astrum.content.raw.internal.recipe;

import jakarta.inject.Provider;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.cache.NegativeCacheFacet;
import org.globalart.astrum.repository.cache.NegativeCacheHandler;
import org.globalart.astrum.repository.http.PartialFetchHandler;
import org.globalart.astrum.repository.httpclient.HttpClientFacet;
import org.globalart.astrum.repository.proxy.ProxyHandler;
import org.globalart.astrum.repository.purge.PurgeUnusedFacet;
import org.globalart.astrum.repository.raw.internal.RawFormat;
import org.globalart.astrum.repository.routing.RoutingRuleHandler;
import org.globalart.astrum.repository.types.ProxyType;
import org.globalart.astrum.repository.view.handlers.ConditionalRequestHandler;
import org.globalart.astrum.repository.view.handlers.HandlerContributor;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.mockito.Mockito.verify;

public class RawProxyRecipeTest
    extends RawRecipeTestSupport
{
  @Mock
  private Repository rawProxyRepository;

  @Mock
  private NegativeCacheHandler negativeCacheHandler;

  @Mock
  private PartialFetchHandler partialFetchHandler;

  @Mock
  private ProxyHandler proxyHandler;

  @Mock
  private ConditionalRequestHandler conditionalRequestHandler;

  @Mock
  private HandlerContributor handlerContributor;

  @Mock
  private RoutingRuleHandler routingRuleHandler;

  @Mock
  private HttpClientFacet httpClientFacet;

  private final Provider<HttpClientFacet> httpClientFacetProvider = () -> httpClientFacet;

  @Mock
  private NegativeCacheFacet negativeCacheFacet;

  private final Provider<NegativeCacheFacet> negativeCacheFacetProvider = () -> negativeCacheFacet;

  @Mock
  private RawProxyFacet proxyFacet;

  private final Provider<RawProxyFacet> rawProxyFacetProvider = () -> proxyFacet;

  @Mock
  private PurgeUnusedFacet purgeUnusedFacet;

  private final Provider<PurgeUnusedFacet> purgeUnusedFacetProvider = () -> purgeUnusedFacet;

  private RawProxyRecipe underTest;

  @Before
  public void setup() {
    underTest =
        new RawProxyRecipe(new ProxyType(), new RawFormat(), httpClientFacetProvider, negativeCacheFacetProvider,
            rawProxyFacetProvider, purgeUnusedFacetProvider, negativeCacheHandler, proxyHandler, routingRuleHandler);
    mockDependencies(underTest);
  }

  @Test
  public void testExpectedFacetsAreAttached() throws Exception {
    underTest.apply(rawProxyRepository);
    verify(rawProxyRepository).attach(securityFacet);
    verify(rawProxyRepository).attach(viewFacet);
    verify(rawProxyRepository).attach(httpClientFacet);
    verify(rawProxyRepository).attach(negativeCacheFacet);
    verify(rawProxyRepository).attach(proxyFacet);
    verify(rawProxyRepository).attach(contentFacet);
    verify(rawProxyRepository).attach(maintenanceFacet);
    verify(rawProxyRepository).attach(searchFacet);
    verify(rawProxyRepository).attach(browseFacet);
    verify(rawProxyRepository).attach(purgeUnusedFacet);
  }
}
