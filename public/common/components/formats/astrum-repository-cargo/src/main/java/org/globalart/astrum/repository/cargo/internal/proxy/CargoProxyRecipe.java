package org.globalart.astrum.repository.cargo.internal.proxy;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.cache.NegativeCacheFacet;
import org.globalart.astrum.repository.cache.NegativeCacheHandler;
import org.globalart.astrum.repository.cargo.CargoFormat;
import org.globalart.astrum.repository.cargo.internal.CargoPaths;
import org.globalart.astrum.repository.cargo.internal.CargoRecipeSupport;
import org.globalart.astrum.repository.proxy.ProxyHandler;
import org.globalart.astrum.repository.purge.PurgeUnusedFacet;
import org.globalart.astrum.repository.routing.RoutingRuleHandler;
import org.globalart.astrum.repository.types.ProxyType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Matcher;
import org.globalart.astrum.repository.view.Route;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;

import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.http.HttpHandlers.notFound;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(CargoProxyRecipe.NAME)
public class CargoProxyRecipe
    extends CargoRecipeSupport
{
  public static final String NAME = "cargo-proxy";

  private final Provider<NegativeCacheFacet> negativeCacheFacet;

  private final Provider<CargoProxyFacet> proxyFacet;

  private final Provider<PurgeUnusedFacet> purgeUnusedFacet;

  private final NegativeCacheHandler negativeCacheHandler;

  private final ProxyHandler proxyHandler;

  private final RoutingRuleHandler routingRuleHandler;

  private final CargoProxySearchHandler searchHandler;

  @Autowired
  public CargoProxyRecipe(
      @Qualifier(ProxyType.NAME) final Type type,
      @Qualifier(CargoFormat.NAME) final Format format,
      final Provider<NegativeCacheFacet> negativeCacheFacet,
      final Provider<CargoProxyFacet> proxyFacet,
      final Provider<PurgeUnusedFacet> purgeUnusedFacet,
      final NegativeCacheHandler negativeCacheHandler,
      final ProxyHandler proxyHandler,
      final RoutingRuleHandler routingRuleHandler,
      final CargoProxySearchHandler searchHandler)
  {
    super(type, format);
    this.negativeCacheFacet = checkNotNull(negativeCacheFacet);
    this.proxyFacet = checkNotNull(proxyFacet);
    this.purgeUnusedFacet = checkNotNull(purgeUnusedFacet);
    this.negativeCacheHandler = checkNotNull(negativeCacheHandler);
    this.proxyHandler = checkNotNull(proxyHandler);
    this.routingRuleHandler = checkNotNull(routingRuleHandler);
    this.searchHandler = checkNotNull(searchHandler);
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
    addCommonRoutes(builder);
    addReadOnlyRoutes(builder);
    builder.route(apiRoute(CargoPaths.search(), searchHandler));
    builder.route(proxyRoute(CargoPaths.download()));
    builder.route(proxyRoute(CargoPaths.index()));
    addBrowseUnsupportedRoute(builder);
    builder.defaultHandlers(notFound());
    facet.configure(builder.create());
    return facet;
  }

  private Route proxyRoute(final Matcher matcher) {
    return new Route.Builder().matcher(matcher)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(routingRuleHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor)
        .handler(conditionalRequestHandler)
        .handler(negativeCacheHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(lastDownloadedHandler)
        .handler(proxyHandler)
        .create();
  }
}
