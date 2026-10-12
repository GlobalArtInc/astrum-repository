package org.globalart.astrum.repository.rubygems.internal.proxy;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.cache.NegativeCacheFacet;
import org.globalart.astrum.repository.cache.NegativeCacheHandler;
import org.globalart.astrum.repository.proxy.ProxyHandler;
import org.globalart.astrum.repository.purge.PurgeUnusedFacet;
import org.globalart.astrum.repository.routing.RoutingRuleHandler;
import org.globalart.astrum.repository.rubygems.RubygemsFormat;
import org.globalart.astrum.repository.rubygems.internal.RubygemsPaths;
import org.globalart.astrum.repository.rubygems.internal.RubygemsRecipeSupport;
import org.globalart.astrum.repository.types.ProxyType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Handler;
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
@Qualifier(RubygemsProxyRecipe.NAME)
public class RubygemsProxyRecipe
    extends RubygemsRecipeSupport
{
  public static final String NAME = "rubygems-proxy";

  private final Provider<NegativeCacheFacet> negativeCacheFacet;

  private final Provider<RubygemsProxyFacet> proxyFacet;

  private final Provider<PurgeUnusedFacet> purgeUnusedFacet;

  private final NegativeCacheHandler negativeCacheHandler;

  private final ProxyHandler proxyHandler;

  private final RoutingRuleHandler routingRuleHandler;

  @Autowired
  public RubygemsProxyRecipe(
      @Qualifier(ProxyType.NAME) final Type type,
      @Qualifier(RubygemsFormat.NAME) final Format format,
      final Provider<NegativeCacheFacet> negativeCacheFacet,
      final Provider<RubygemsProxyFacet> proxyFacet,
      final Provider<PurgeUnusedFacet> purgeUnusedFacet,
      final NegativeCacheHandler negativeCacheHandler,
      final ProxyHandler proxyHandler,
      final RoutingRuleHandler routingRuleHandler)
  {
    super(type, format);
    this.negativeCacheFacet = checkNotNull(negativeCacheFacet);
    this.proxyFacet = checkNotNull(proxyFacet);
    this.purgeUnusedFacet = checkNotNull(purgeUnusedFacet);
    this.negativeCacheHandler = checkNotNull(negativeCacheHandler);
    this.proxyHandler = checkNotNull(proxyHandler);
    this.routingRuleHandler = checkNotNull(routingRuleHandler);
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
    builder.route(proxyRoute(RubygemsPaths.versions(), indexEtagHandler));
    builder.route(proxyRoute(RubygemsPaths.names(), indexEtagHandler));
    builder.route(proxyRoute(RubygemsPaths.info(), indexEtagHandler));
    builder.route(proxyRoute(RubygemsPaths.legacySpecs()));
    builder.route(proxyRoute(RubygemsPaths.quickSpec()));
    builder.route(proxyRoute(RubygemsPaths.gem()));
    addBrowseUnsupportedRoute(builder);
    builder.defaultHandlers(notFound());
    facet.configure(builder.create());
    return facet;
  }

  private Route proxyRoute(final Matcher matcher, final Handler... extra) {
    Route.Builder route = new Route.Builder().matcher(matcher)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(routingRuleHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor)
        .handler(conditionalRequestHandler)
        .handler(negativeCacheHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler);
    for (Handler handler : extra) {
      route.handler(handler);
    }
    return route
        .handler(lastDownloadedHandler)
        .handler(proxyHandler)
        .create();
  }
}
