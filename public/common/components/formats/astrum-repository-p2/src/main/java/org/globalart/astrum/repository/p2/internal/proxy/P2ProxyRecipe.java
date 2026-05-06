
package org.globalart.astrum.repository.p2.internal.proxy;

import java.util.Map;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.RecipeSupport;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.cache.NegativeCacheFacet;
import org.globalart.astrum.repository.cache.NegativeCacheHandler;
import org.globalart.astrum.repository.content.browse.BrowseFacet;
import org.globalart.astrum.repository.content.maintenance.LastAssetMaintenanceFacet;
import org.globalart.astrum.repository.content.search.SearchFacet;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.http.PartialFetchHandler;
import org.globalart.astrum.repository.httpclient.HttpClientFacet;
import org.globalart.astrum.repository.p2.P2ContentFacet;
import org.globalart.astrum.repository.p2.internal.AssetKind;
import org.globalart.astrum.repository.p2.internal.P2Format;
import org.globalart.astrum.repository.p2.internal.security.P2SecurityFacet;
import org.globalart.astrum.repository.p2.internal.util.P2PathUtils;
import org.globalart.astrum.repository.proxy.ProxyHandler;
import org.globalart.astrum.repository.purge.PurgeUnusedFacet;
import org.globalart.astrum.repository.routing.RoutingRuleHandler;
import org.globalart.astrum.repository.security.SecurityHandler;
import org.globalart.astrum.repository.types.ProxyType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Matcher;
import org.globalart.astrum.repository.view.Route;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;
import org.globalart.astrum.repository.view.handlers.ConditionalRequestHandler;
import org.globalart.astrum.repository.view.handlers.ContentHeadersHandler;
import org.globalart.astrum.repository.view.handlers.ExceptionHandler;
import org.globalart.astrum.repository.view.handlers.HandlerContributor;
import org.globalart.astrum.repository.view.handlers.LastDownloadedHandler;
import org.globalart.astrum.repository.view.handlers.TimingHandler;
import org.globalart.astrum.repository.view.matchers.ActionMatcher;
import org.globalart.astrum.repository.view.matchers.logic.LogicMatchers;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher;

import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(P2ProxyRecipe.NAME)
public class P2ProxyRecipe
    extends RecipeSupport
{
  public static final String NAME = "p2-proxy";

  private final Provider<ConfigurableViewFacet> viewFacet;

  private final Provider<HttpClientFacet> httpClientFacet;

  private final Provider<NegativeCacheFacet> negativeCacheFacet;

  private final Provider<P2ContentFacet> contentFacet;

  private final Provider<P2ProxyCacheInvalidatorFacet> cacheInvalidatorFacet;

  private final Provider<P2ProxyFacet> proxyFacet;

  private final Provider<P2SecurityFacet> securityFacet;

  private final Provider<PurgeUnusedFacet> purgeUnusedFacet;

  private final Provider<SearchFacet> searchFacet;

  private final Provider<LastAssetMaintenanceFacet> maintenanceFacet;

  private final Provider<BrowseFacet> browseFacet;

  private final ConditionalRequestHandler conditionalRequestHandler;

  private final ContentHeadersHandler contentHeadersHandler;

  private final ExceptionHandler exceptionHandler;

  private final HandlerContributor handlerContributor;

  private final LastDownloadedHandler lastDownloadedHandler;

  private final NegativeCacheHandler negativeCacheHandler;

  private final PartialFetchHandler partialFetchHandler;

  private final ProxyHandler proxyHandler;

  private final RoutingRuleHandler routingRuleHandler;

  private final SecurityHandler securityHandler;

  private final TimingHandler timingHandler;

  @Autowired
  public P2ProxyRecipe(
      @Qualifier(ProxyType.NAME) final Type type,
      @Qualifier(P2Format.NAME) final Format format,
      final Provider<ConfigurableViewFacet> viewFacet,
      final Provider<HttpClientFacet> httpClientFacet,
      final Provider<NegativeCacheFacet> negativeCacheFacet,
      final Provider<P2ContentFacet> contentFacet,
      final Provider<P2ProxyCacheInvalidatorFacet> cacheInvalidatorFacet,
      final Provider<P2ProxyFacet> proxyFacet,
      final Provider<P2SecurityFacet> securityFacet,
      final Provider<PurgeUnusedFacet> purgeUnusedFacet,
      final Provider<SearchFacet> searchFacet,
      final Provider<LastAssetMaintenanceFacet> maintenanceFacet,
      final Provider<BrowseFacet> browseFacet,
      final ConditionalRequestHandler conditionalRequestHandler,
      final ContentHeadersHandler contentHeadersHandler,
      final ExceptionHandler exceptionHandler,
      final HandlerContributor handlerContributor,
      final LastDownloadedHandler lastDownloadedHandler,
      final NegativeCacheHandler negativeCacheHandler,
      final PartialFetchHandler partialFetchHandler,
      final ProxyHandler proxyHandler,
      final RoutingRuleHandler routingRuleHandler,
      final SecurityHandler securityHandler,
      final TimingHandler timingHandler)
  {
    super(type, format);
    this.viewFacet = checkNotNull(viewFacet);
    this.httpClientFacet = checkNotNull(httpClientFacet);
    this.negativeCacheFacet = checkNotNull(negativeCacheFacet);
    this.contentFacet = checkNotNull(contentFacet);
    this.cacheInvalidatorFacet = checkNotNull(cacheInvalidatorFacet);
    this.proxyFacet = checkNotNull(proxyFacet);
    this.securityFacet = checkNotNull(securityFacet);
    this.purgeUnusedFacet = checkNotNull(purgeUnusedFacet);
    this.searchFacet = checkNotNull(searchFacet);
    this.maintenanceFacet = checkNotNull(maintenanceFacet);
    this.browseFacet = checkNotNull(browseFacet);
    this.conditionalRequestHandler = checkNotNull(conditionalRequestHandler);
    this.contentHeadersHandler = checkNotNull(contentHeadersHandler);
    this.exceptionHandler = checkNotNull(exceptionHandler);
    this.handlerContributor = checkNotNull(handlerContributor);
    this.lastDownloadedHandler = checkNotNull(lastDownloadedHandler);
    this.negativeCacheHandler = checkNotNull(negativeCacheHandler);
    this.partialFetchHandler = checkNotNull(partialFetchHandler);
    this.proxyHandler = checkNotNull(proxyHandler);
    this.routingRuleHandler = checkNotNull(routingRuleHandler);
    this.securityHandler = checkNotNull(securityHandler);
    this.timingHandler = checkNotNull(timingHandler);
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
    repository.attach(purgeUnusedFacet.get());
    repository.attach(cacheInvalidatorFacet.get());
    repository.attach(browseFacet.get());
  }

  static Matcher matcher(final String pattern, final AssetKind assetKind) {
    return LogicMatchers.and(
        new ActionMatcher(GET, HEAD),
        new TokenMatcher(pattern),
        context -> {
          context.getAttributes().set(AssetKind.class, assetKind);
          return true;
        });
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();

    for (String prefix : new String[]{P2PathUtils.SITE_PREFIX, ""}) {
      for (Map.Entry<String, AssetKind> entry : P2PathUtils.patterns(prefix).entrySet()) {
        builder.route(route(matcher(entry.getKey(), entry.getValue())));
      }
    }

    addBrowseUnsupportedRoute(builder);

    builder.defaultHandlers(HttpHandlers.notFound());

    facet.configure(builder.create());

    return facet;
  }

  private Route route(final Matcher matcher) {
    return new Route.Builder().matcher(matcher)
        .handler(timingHandler)
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
