
package org.globalart.astrum.repository.alpine.internal.recipe;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.alpine.AlpineFormat;
import org.globalart.astrum.repository.alpine.AssetKind;
import org.globalart.astrum.repository.alpine.internal.proxy.AlpineProxyFacet;
import org.globalart.astrum.repository.cache.NegativeCacheFacet;
import org.globalart.astrum.repository.cache.NegativeCacheHandler;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.httpclient.HttpClientFacet;
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
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(AlpineProxyRecipe.NAME)
public class AlpineProxyRecipe
    extends AlpineRecipeSupport
{
  public static final String NAME = "alpine-proxy";

  private final Provider<AlpineProxyFacet> proxyFacet;

  private final Provider<HttpClientFacet> httpClientFacet;

  private final Provider<NegativeCacheFacet> negativeCacheFacet;

  private final Provider<PurgeUnusedFacet> purgeUnusedFacet;

  private final NegativeCacheHandler negativeCacheHandler;

  private final RoutingRuleHandler routingRuleHandler;

  private final ProxyHandler proxyHandler;

  @Autowired
  public AlpineProxyRecipe(
      @Qualifier(ProxyType.NAME) final Type type,
      @Qualifier(AlpineFormat.NAME) final Format format,
      final Provider<AlpineProxyFacet> proxyFacet,
      final Provider<HttpClientFacet> httpClientFacet,
      final Provider<NegativeCacheFacet> negativeCacheFacet,
      final Provider<PurgeUnusedFacet> purgeUnusedFacet,
      final NegativeCacheHandler negativeCacheHandler,
      final RoutingRuleHandler routingRuleHandler,
      final ProxyHandler proxyHandler)
  {
    super(type, format);
    this.proxyFacet = checkNotNull(proxyFacet);
    this.httpClientFacet = checkNotNull(httpClientFacet);
    this.negativeCacheFacet = checkNotNull(negativeCacheFacet);
    this.purgeUnusedFacet = checkNotNull(purgeUnusedFacet);
    this.negativeCacheHandler = checkNotNull(negativeCacheHandler);
    this.routingRuleHandler = checkNotNull(routingRuleHandler);
    this.proxyHandler = checkNotNull(proxyHandler);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(securityFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(contentFacet.get());
    repository.attach(httpClientFacet.get());
    repository.attach(negativeCacheFacet.get());
    repository.attach(proxyFacet.get());
    repository.attach(maintenanceFacet.get());
    repository.attach(searchFacet.get());
    repository.attach(browseFacet.get());
    repository.attach(purgeUnusedFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();

    builder.route(proxy(indexMatcher(), AssetKind.APK_INDEX));
    builder.route(proxy(packageMatcher(GET, HEAD), AssetKind.PACKAGE));

    addBrowseUnsupportedRoute(builder);

    builder.route(proxy(otherMatcher(), AssetKind.OTHER));

    builder.defaultHandlers(HttpHandlers.notFound());

    facet.configure(builder.create());

    return facet;
  }

  private Route proxy(final Matcher matcher, final AssetKind assetKind) {
    return new Route.Builder().matcher(matcher)
        .handler(timingHandler)
        .handler(assetKindHandler(assetKind))
        .handler(securityHandler)
        .handler(routingRuleHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor)
        .handler(negativeCacheHandler)
        .handler(conditionalRequestHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(lastDownloadedHandler)
        .handler(proxyHandler)
        .create();
  }
}
