
package org.globalart.astrum.content.maven.internal.recipe;

import jakarta.inject.Provider;

import org.globalart.astrum.content.maven.internal.index.MavenContentProxyIndexFacet;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.cache.NegativeCacheFacet;
import org.globalart.astrum.repository.cache.NegativeCacheHandler;
import org.globalart.astrum.repository.httpclient.HttpClientFacet;
import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.globalart.astrum.repository.proxy.ProxyHandler;
import org.globalart.astrum.repository.purge.PurgeUnusedFacet;
import org.globalart.astrum.repository.types.ProxyType;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.mockito.Mockito.verify;

public class MavenProxyRecipeTest
    extends MavenRecipeTestSupport
{
  @Mock
  private Repository mavenProxyRepository;

  @Mock
  private NegativeCacheHandler negativeCacheHandler;

  @Mock
  private ProxyHandler proxyHandler;

  @Mock
  private HttpClientFacet httpClientFacet;

  private final Provider<HttpClientFacet> httpClientFacetProvider = () -> httpClientFacet;

  @Mock
  private NegativeCacheFacet negativeCacheFacet;

  private final Provider<NegativeCacheFacet> negativeCacheFacetProvider = () -> negativeCacheFacet;

  @Mock
  private MavenProxyFacet mavenProxyFacet;

  private final Provider<MavenProxyFacet> mavenProxyFacetProvider = () -> mavenProxyFacet;

  @Mock
  private PurgeUnusedFacet purgeUnusedFacet;

  private final Provider<PurgeUnusedFacet> purgeUnusedFacetProvider = () -> purgeUnusedFacet;

  @Mock
  private MavenContentProxyIndexFacet mavenContentProxyIndexFacet;

  private final Provider<MavenContentProxyIndexFacet> mavenContentProxyIndexFacetProvider =
      () -> mavenContentProxyIndexFacet;

  private MavenProxyRecipe underTest;

  @Before
  public void setup() {
    underTest = new MavenProxyRecipe(new ProxyType(), new Maven2Format(), httpClientFacetProvider,
        negativeCacheFacetProvider, mavenProxyFacetProvider, purgeUnusedFacetProvider, negativeCacheHandler,
        proxyHandler, mavenContentProxyIndexFacetProvider);
    mockHandlers(underTest);
    mockFacets(underTest);
  }

  @Test
  public void testExpectedFacetsAreAttached() throws Exception {
    underTest.apply(mavenProxyRepository);
    verify(mavenProxyRepository).attach(securityFacet);
    verify(mavenProxyRepository).attach(viewFacet);
    verify(mavenProxyRepository).attach(httpClientFacet);
    verify(mavenProxyRepository).attach(negativeCacheFacet);
    verify(mavenProxyRepository).attach(mavenProxyFacet);
    verify(mavenProxyRepository).attach(mavenContentFacet);
    verify(mavenProxyRepository).attach(purgeUnusedFacet);
    verify(mavenProxyRepository).attach(searchFacet);
    verify(mavenProxyRepository).attach(browseFacet);
    verify(mavenProxyRepository).attach(mavenContentProxyIndexFacet);
    verify(mavenProxyRepository).attach(mavenMaintenanceFacet);
    verify(mavenProxyRepository).attach(removeSnapshotsFacet);
  }
}
