package org.globalart.astrum.repository.docker.internal.recipe;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.cache.NegativeCacheFacet;
import org.globalart.astrum.repository.cache.NegativeCacheHandler;
import org.globalart.astrum.repository.docker.DockerFormat;
import org.globalart.astrum.repository.docker.internal.proxy.DockerProxyFacet;
import org.globalart.astrum.repository.docker.internal.proxy.DockerProxyHandler;
import org.globalart.astrum.repository.docker.internal.proxy.DockerProxyReferrersHandler;
import org.globalart.astrum.repository.docker.internal.proxy.DockerProxyTagsHandler;
import org.globalart.astrum.repository.docker.internal.v2.DockerRoute;
import org.globalart.astrum.repository.docker.internal.v2.DockerRouteMatcher;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.httpclient.HttpClientFacet;
import org.globalart.astrum.repository.purge.PurgeUnusedFacet;
import org.globalart.astrum.repository.routing.RoutingRuleHandler;
import org.globalart.astrum.repository.types.ProxyType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;

import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(DockerProxyRecipe.NAME)
public class DockerProxyRecipe
    extends DockerRecipeSupport
{
  public static final String NAME = "docker-proxy";

  private final Provider<HttpClientFacet> httpClientFacet;

  private final Provider<NegativeCacheFacet> negativeCacheFacet;

  private final Provider<DockerProxyFacet> proxyFacet;

  private final Provider<PurgeUnusedFacet> purgeUnusedFacet;

  private final NegativeCacheHandler negativeCacheHandler;

  private final RoutingRuleHandler routingRuleHandler;

  private final Provider<DockerProxyHandler> proxyHandler;

  private final DockerProxyTagsHandler tagsHandler;

  private final DockerProxyReferrersHandler referrersHandler;

  @Autowired
  public DockerProxyRecipe(
      @Qualifier(ProxyType.NAME) final Type type,
      @Qualifier(DockerFormat.NAME) final Format format,
      final Provider<HttpClientFacet> httpClientFacet,
      final Provider<NegativeCacheFacet> negativeCacheFacet,
      final Provider<DockerProxyFacet> proxyFacet,
      final Provider<PurgeUnusedFacet> purgeUnusedFacet,
      final NegativeCacheHandler negativeCacheHandler,
      final RoutingRuleHandler routingRuleHandler,
      final Provider<DockerProxyHandler> proxyHandler,
      final DockerProxyTagsHandler tagsHandler,
      final DockerProxyReferrersHandler referrersHandler)
  {
    super(type, format);
    this.httpClientFacet = checkNotNull(httpClientFacet);
    this.negativeCacheFacet = checkNotNull(negativeCacheFacet);
    this.proxyFacet = checkNotNull(proxyFacet);
    this.purgeUnusedFacet = checkNotNull(purgeUnusedFacet);
    this.negativeCacheHandler = checkNotNull(negativeCacheHandler);
    this.routingRuleHandler = checkNotNull(routingRuleHandler);
    this.proxyHandler = checkNotNull(proxyHandler);
    this.tagsHandler = checkNotNull(tagsHandler);
    this.referrersHandler = checkNotNull(referrersHandler);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(contentFacet.get());
    repository.attach(securityFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(httpClientFacet.get());
    repository.attach(negativeCacheFacet.get());
    repository.attach(proxyFacet.get());
    repository.attach(maintenanceFacet.get());
    repository.attach(searchFacet.get());
    repository.attach(browseFacet.get());
    repository.attach(purgeUnusedFacet.get());
    repository.attach(connectorFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();
    DockerProxyHandler handler = proxyHandler.get();

    metadataRoutes(builder, tagsHandler, metadataHandler, referrersHandler);
    builder.route(route(DockerRoute.Type.MANIFEST, DockerRouteMatcher.MANIFEST, readActions(),
        routingRuleHandler, negativeCacheHandler, lastDownloadedHandler, handler));
    builder.route(route(DockerRoute.Type.BLOB, DockerRouteMatcher.BLOB, readActions(),
        routingRuleHandler, negativeCacheHandler, partialFetchHandler, lastDownloadedHandler, handler));
    builder.route(unsupportedRoute());

    addBrowseUnsupportedRoute(builder);

    builder.defaultHandlers(HttpHandlers.notFound());

    facet.configure(builder.create());
    return facet;
  }
}
