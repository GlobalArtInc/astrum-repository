package org.globalart.astrum.content.npm.internal;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.cache.NegativeCacheFacet;
import org.globalart.astrum.repository.content.maintenance.LastAssetMaintenanceFacet;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.httpclient.HttpClientFacet;
import org.globalart.astrum.repository.npm.internal.NpmFormat;
import org.globalart.astrum.repository.npm.internal.NpmHandlers;
import org.globalart.astrum.repository.npm.internal.NpmNegativeCacheHandler;
import org.globalart.astrum.repository.npm.internal.NpmProxyCacheInvalidatorFacetImpl;
import org.globalart.astrum.repository.npm.internal.NpmProxyFacet.ProxyTarget;
import org.globalart.astrum.repository.npm.internal.NpmProxyHandler;
import org.globalart.astrum.repository.npm.internal.search.v1.NpmSearchFacetProxy;
import org.globalart.astrum.repository.purge.PurgeUnusedFacet;
import org.globalart.astrum.repository.types.ProxyType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;

import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;
import static org.globalart.astrum.repository.npm.internal.NpmPaths.distTagsMatcher;
import static org.globalart.astrum.repository.npm.internal.NpmPaths.packageMatcher;
import static org.globalart.astrum.repository.npm.internal.NpmPaths.searchV1Matcher;
import static org.globalart.astrum.repository.npm.internal.NpmPaths.tarballMatcher;
import static org.globalart.astrum.repository.npm.internal.NpmProxyFacet.ProxyTarget.DIST_TAGS;
import static org.globalart.astrum.repository.npm.internal.NpmProxyFacet.ProxyTarget.PACKAGE;
import static org.globalart.astrum.repository.npm.internal.NpmProxyFacet.ProxyTarget.SEARCH_V1_RESULTS;
import static org.globalart.astrum.repository.npm.internal.NpmProxyFacet.ProxyTarget.TARBALL;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(NpmProxyRecipe.NAME)
public class NpmProxyRecipe
    extends NpmRecipeSupport
{
  public static final String NAME = "npm-proxy";

  private final Provider<HttpClientFacet> httpClientFacet;

  private final Provider<NegativeCacheFacet> negativeCacheFacet;

  private final Provider<NpmContentProxyFacet> proxyFacet;

  private final Provider<NpmSearchFacetProxy> searchV1Facet;

  private final Provider<PurgeUnusedFacet> purgeUnusedFacet;

  private final Provider<LastAssetMaintenanceFacet> maintenanceFacet;

  private final Provider<NpmProxyCacheInvalidatorFacetImpl> cacheInvalidatorFacet;

  private final NpmNegativeCacheHandler negativeCacheHandler;

  private final NpmProxyHandler proxyHandler;

  @Autowired
  public NpmProxyRecipe(
      @Qualifier(ProxyType.NAME) final Type type,
      @Qualifier(NpmFormat.NAME) final Format format,
      final Provider<HttpClientFacet> httpClientFacet,
      final Provider<NegativeCacheFacet> negativeCacheFacet,
      final Provider<NpmContentProxyFacet> proxyFacet,
      final Provider<NpmSearchFacetProxy> searchV1Facet,
      final Provider<PurgeUnusedFacet> purgeUnusedFacet,
      final Provider<LastAssetMaintenanceFacet> maintenanceFacet,
      final Provider<NpmProxyCacheInvalidatorFacetImpl> cacheInvalidatorFacet,
      final NpmNegativeCacheHandler negativeCacheHandler,
      final NpmProxyHandler proxyHandler)
  {
    super(type, format);
    this.httpClientFacet = checkNotNull(httpClientFacet);
    this.negativeCacheFacet = checkNotNull(negativeCacheFacet);
    this.proxyFacet = checkNotNull(proxyFacet);
    this.searchV1Facet = checkNotNull(searchV1Facet);
    this.purgeUnusedFacet = checkNotNull(purgeUnusedFacet);
    this.maintenanceFacet = checkNotNull(maintenanceFacet);
    this.cacheInvalidatorFacet = checkNotNull(cacheInvalidatorFacet);
    this.negativeCacheHandler = checkNotNull(negativeCacheHandler);
    this.proxyHandler = checkNotNull(proxyHandler);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(securityFacet.get());
    repository.attach(tokenFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(httpClientFacet.get());
    repository.attach(negativeCacheFacet.get());
    repository.attach(contentFacet.get());
    repository.attach(proxyFacet.get());
    repository.attach(searchV1Facet.get());
    repository.attach(maintenanceFacet.get());
    repository.attach(searchFacet.get());
    repository.attach(browseFacet.get());
    repository.attach(purgeUnusedFacet.get());
    repository.attach(cacheInvalidatorFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();

    addBrowseUnsupportedRoute(builder);

    builder.route(searchV1Matcher()
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(NpmHandlers.npmErrorHandler)
        .handler(partialFetchHandler)
        .handler(conditionalRequestHandler)
        .handler(contentHeadersHandler)
        .handler(proxyTargetHandler(SEARCH_V1_RESULTS))
        .handler(NpmHandlers.searchV1)
        .create());

    createCommonRoutes(builder);

    builder.route(packageMatcher(GET, HEAD)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(routingHandler)
        .handler(NpmHandlers.npmErrorHandler)
        .handler(negativeCacheHandler)
        .handler(partialFetchHandler)
        .handler(conditionalRequestHandler)
        .handler(contentHeadersHandler)
        .handler(proxyTargetHandler(PACKAGE))
        .handler(lastDownloadedHandler)
        .handler(proxyHandler)
        .create());

    builder.route(tarballMatcher(GET, HEAD)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(routingHandler)
        .handler(NpmHandlers.npmErrorHandler)
        .handler(handlerContributor)
        .handler(negativeCacheHandler)
        .handler(partialFetchHandler)
        .handler(conditionalRequestHandler)
        .handler(contentHeadersHandler)
        .handler(proxyTargetHandler(TARBALL))
        .handler(lastDownloadedHandler)
        .handler(proxyHandler)
        .create());

    builder.route(distTagsMatcher(GET)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(routingHandler)
        .handler(NpmHandlers.npmErrorHandler)
        .handler(handlerContributor)
        .handler(contentHeadersHandler)
        .handler(proxyTargetHandler(DIST_TAGS))
        .handler(proxyHandler)
        .create());

    builder.defaultHandlers(HttpHandlers.notFound());

    facet.configure(builder.create());

    return facet;
  }

  private static Handler proxyTargetHandler(final ProxyTarget value) {
    return context -> {
      context.getAttributes().set(ProxyTarget.class, value);
      return context.proceed();
    };
  }
}
