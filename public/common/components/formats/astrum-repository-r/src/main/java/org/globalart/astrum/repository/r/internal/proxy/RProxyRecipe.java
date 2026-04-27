
package org.globalart.astrum.repository.r.internal.proxy;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.cache.NegativeCacheFacet;
import org.globalart.astrum.repository.cache.NegativeCacheHandler;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.httpclient.HttpClientFacet;
import org.globalart.astrum.repository.proxy.ProxyHandler;
import org.globalart.astrum.repository.purge.PurgeUnusedFacet;
import org.globalart.astrum.repository.r.AssetKind;
import org.globalart.astrum.repository.r.RFormat;
import org.globalart.astrum.repository.r.internal.RRecipeSupport;
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
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(RProxyRecipe.NAME)
public class RProxyRecipe
    extends RRecipeSupport
{
  public static final String NAME = "r-proxy";

  private final Provider<RProxyFacet> proxyFacet;

  private final Provider<HttpClientFacet> httpClientFacet;

  private final Provider<NegativeCacheFacet> negativeCacheFacet;

  private final Provider<PurgeUnusedFacet> purgeUnusedFacet;

  private final NegativeCacheHandler negativeCacheHandler;

  private final ProxyHandler proxyHandler;

  private final RoutingRuleHandler routingRuleHandler;

  @Autowired
  public RProxyRecipe(
      @Qualifier(ProxyType.NAME) final Type type,
      @Qualifier(RFormat.NAME) final Format format,
      final Provider<RProxyFacet> proxyFacet,
      final Provider<HttpClientFacet> httpClientFacet,
      final Provider<NegativeCacheFacet> negativeCacheFacet,
      final Provider<PurgeUnusedFacet> purgeUnusedFacet,
      final NegativeCacheHandler negativeCacheHandler,
      final ProxyHandler proxyHandler,
      final RoutingRuleHandler routingRuleHandler)
  {
    super(type, format);
    this.proxyFacet = checkNotNull(proxyFacet);
    this.httpClientFacet = checkNotNull(httpClientFacet);
    this.negativeCacheFacet = checkNotNull(negativeCacheFacet);
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

    builder.route(proxyRoute(allPackagesMatcher(), AssetKind.PACKAGES, false));
    builder.route(proxyRoute(rdsMatcher(), AssetKind.RDS_METADATA, false));
    builder.route(proxyRoute(archiveMatcher(GET, HEAD), AssetKind.ARCHIVE, true));

    addBrowseUnsupportedRoute(builder);

    builder.defaultHandlers(HttpHandlers.notFound());

    facet.configure(builder.create());

    return facet;
  }

  private Route proxyRoute(final Matcher matcher, final AssetKind assetKind, final boolean archive) {
    Route.Builder route = new Route.Builder().matcher(matcher)
        .handler(timingHandler)
        .handler(assetKindHandler(assetKind))
        .handler(securityHandler)
        .handler(routingRuleHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor)
        .handler(negativeCacheHandler)
        .handler(conditionalRequestHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler);
    if (archive) {
      route.handler(lastDownloadedHandler);
    }
    return route.handler(proxyHandler).create();
  }
}
