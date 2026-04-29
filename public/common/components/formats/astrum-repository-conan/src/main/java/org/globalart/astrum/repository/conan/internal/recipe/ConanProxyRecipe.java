
package org.globalart.astrum.repository.conan.internal.recipe;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.cache.NegativeCacheFacet;
import org.globalart.astrum.repository.cache.NegativeCacheHandler;
import org.globalart.astrum.repository.conan.ConanFormat;
import org.globalart.astrum.repository.conan.internal.ConanEndpoint;
import org.globalart.astrum.repository.conan.internal.common.ConanUserHandlers;
import org.globalart.astrum.repository.conan.internal.proxy.ConanProxyFacet;
import org.globalart.astrum.repository.conan.internal.proxy.ConanProxyHandlers;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.proxy.ProxyHandler;
import org.globalart.astrum.repository.purge.PurgeUnusedFacet;
import org.globalart.astrum.repository.routing.RoutingRuleHandler;
import org.globalart.astrum.repository.types.ProxyType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Route;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;

import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(ConanProxyRecipe.NAME)
public class ConanProxyRecipe
    extends ConanRecipeSupport
{
  public static final String NAME = "conan-proxy";

  private final Provider<NegativeCacheFacet> negativeCacheFacet;

  private final Provider<ConanProxyFacet> proxyFacet;

  private final Provider<PurgeUnusedFacet> purgeUnusedFacet;

  private final NegativeCacheHandler negativeCacheHandler;

  private final ProxyHandler proxyHandler;

  private final RoutingRuleHandler routingRuleHandler;

  private final ConanProxyHandlers proxyHandlers;

  @Autowired
  public ConanProxyRecipe(
      @Qualifier(ProxyType.NAME) final Type type,
      @Qualifier(ConanFormat.NAME) final Format format,
      final Provider<NegativeCacheFacet> negativeCacheFacet,
      final Provider<ConanProxyFacet> proxyFacet,
      final Provider<PurgeUnusedFacet> purgeUnusedFacet,
      final NegativeCacheHandler negativeCacheHandler,
      final ProxyHandler proxyHandler,
      final RoutingRuleHandler routingRuleHandler,
      final ConanProxyHandlers proxyHandlers)
  {
    super(type, format);
    this.negativeCacheFacet = checkNotNull(negativeCacheFacet);
    this.proxyFacet = checkNotNull(proxyFacet);
    this.purgeUnusedFacet = checkNotNull(purgeUnusedFacet);
    this.negativeCacheHandler = checkNotNull(negativeCacheHandler);
    this.proxyHandler = checkNotNull(proxyHandler);
    this.routingRuleHandler = checkNotNull(routingRuleHandler);
    this.proxyHandlers = checkNotNull(proxyHandlers);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(securityFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(httpClientFacet.get());
    repository.attach(negativeCacheFacet.get());
    repository.attach(maintenanceFacet.get());
    repository.attach(proxyFacet.get());
    repository.attach(contentFacet.get());
    repository.attach(searchFacet.get());
    repository.attach(browseFacet.get());
    repository.attach(purgeUnusedFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();
    addUserRoutes(builder, ConanProxyRecipe::capabilities);
    for (ConanEndpoint endpoint : ConanEndpoint.values()) {
      if (isUserEndpoint(endpoint) || !endpoint.isRead()) {
        continue;
      }
      if (isSearch(endpoint)) {
        builder.route(simpleRoute(endpoint, proxyHandlers.protocol, proxyHandlers.search));
      }
      else if (endpoint.assetKind() != null) {
        builder.route(proxyRoute(endpoint));
      }
    }
    addBrowseUnsupportedRoute(builder);
    builder.defaultHandlers(HttpHandlers.notFound());
    facet.configure(builder.create());
    return facet;
  }

  private Route proxyRoute(final ConanEndpoint endpoint) {
    return new Route.Builder().matcher(endpoint.matcher())
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(routingRuleHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor)
        .handler(proxyHandlers.protocol)
        .handler(negativeCacheHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(lastDownloadedHandler)
        .handler(proxyHandler)
        .create();
  }

  private static String capabilities(final Repository repository) {
    return repository.facet(ConanProxyFacet.class).isV1() ? "" : ConanUserHandlers.REVISIONS;
  }
}
