package org.globalart.astrum.repository.swift.internal.proxy;

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
import org.globalart.astrum.repository.swift.SwiftFormat;
import org.globalart.astrum.repository.swift.internal.SwiftPaths;
import org.globalart.astrum.repository.swift.internal.SwiftRecipeSupport;
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

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(SwiftProxyRecipe.NAME)
public class SwiftProxyRecipe
    extends SwiftRecipeSupport
{
  public static final String NAME = "swift-proxy";

  private final Provider<NegativeCacheFacet> negativeCacheFacet;

  private final Provider<SwiftProxyFacet> proxyFacet;

  private final Provider<PurgeUnusedFacet> purgeUnusedFacet;

  private final NegativeCacheHandler negativeCacheHandler;

  private final ProxyHandler proxyHandler;

  private final RoutingRuleHandler routingRuleHandler;

  private final SwiftProxyHandlers proxyHandlers;

  @Autowired
  public SwiftProxyRecipe(
      @Qualifier(ProxyType.NAME) final Type type,
      @Qualifier(SwiftFormat.NAME) final Format format,
      final Provider<NegativeCacheFacet> negativeCacheFacet,
      final Provider<SwiftProxyFacet> proxyFacet,
      final Provider<PurgeUnusedFacet> purgeUnusedFacet,
      final NegativeCacheHandler negativeCacheHandler,
      final ProxyHandler proxyHandler,
      final RoutingRuleHandler routingRuleHandler,
      final SwiftProxyHandlers proxyHandlers)
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
    addCommonRoutes(builder);
    builder.route(apiRoute(SwiftPaths.identifiers(), proxyHandlers.identifiers));
    builder.route(apiRoute(SwiftPaths.publish(), proxyHandlers.publish));
    builder.route(proxyRoute(SwiftPaths.archive(), proxyHandlers.archive));
    builder.route(proxyRoute(SwiftPaths.manifest(), proxyHandlers.manifest));
    builder.route(proxyRoute(SwiftPaths.releases(), proxyHandlers.releases));
    builder.route(proxyRoute(SwiftPaths.release(), proxyHandlers.release));
    addDefaultRoutes(builder);
    facet.configure(builder.create());
    return facet;
  }

  private Route proxyRoute(final Matcher matcher, final Handler formatHandler) {
    return new Route.Builder().matcher(matcher)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(routingRuleHandler)
        .handler(exceptionHandler)
        .handler(apiHandler)
        .handler(handlerContributor)
        .handler(conditionalRequestHandler)
        .handler(negativeCacheHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(lastDownloadedHandler)
        .handler(formatHandler)
        .handler(proxyHandler)
        .create();
  }
}
