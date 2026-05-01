
package org.globalart.astrum.repository.cocoapods.internal.proxy;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.RecipeSupport;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.cache.NegativeCacheFacet;
import org.globalart.astrum.repository.cache.NegativeCacheHandler;
import org.globalart.astrum.repository.cocoapods.CocoapodsContentFacet;
import org.globalart.astrum.repository.cocoapods.internal.AssetKind;
import org.globalart.astrum.repository.cocoapods.internal.CocoapodsFormat;
import org.globalart.astrum.repository.cocoapods.internal.CocoapodsSecurityFacet;
import org.globalart.astrum.repository.content.browse.BrowseFacet;
import org.globalart.astrum.repository.content.maintenance.LastAssetMaintenanceFacet;
import org.globalart.astrum.repository.content.search.SearchFacet;
import org.globalart.astrum.repository.http.PartialFetchHandler;
import org.globalart.astrum.repository.httpclient.HttpClientFacet;
import org.globalart.astrum.repository.proxy.ProxyHandler;
import org.globalart.astrum.repository.purge.PurgeUnusedFacet;
import org.globalart.astrum.repository.routing.RoutingRuleHandler;
import org.globalart.astrum.repository.security.SecurityHandler;
import org.globalart.astrum.repository.types.ProxyType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Handler;
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
import static org.globalart.astrum.repository.cocoapods.internal.AssetKind.CDN_METADATA;
import static org.globalart.astrum.repository.cocoapods.internal.AssetKind.POD;
import static org.globalart.astrum.repository.cocoapods.internal.AssetKind.SPEC;
import static org.globalart.astrum.repository.cocoapods.internal.CocoapodsFormat.FILENAME_KEY;
import static org.globalart.astrum.repository.cocoapods.internal.CocoapodsFormat.PACKAGE_NAME_KEY;
import static org.globalart.astrum.repository.cocoapods.internal.CocoapodsFormat.PACKAGE_VERSION_KEY;
import static org.globalart.astrum.repository.http.HttpHandlers.notFound;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(CocoapodsProxyRecipe.NAME)
public class CocoapodsProxyRecipe
    extends RecipeSupport
{
  public static final String NAME = "cocoapods-proxy";

  public static final String[] CDN_METADATA_PATTERNS = {
      "/CocoaPods-version.yml",
      "/deprecated_podspecs.txt",
      "/all_pods.txt",
      "/all_pods_versions{shard:(_[0-9a-f]+)*}.txt",
      "/AlgoliaSearch.yml"
  };

  public static final String SPEC_PATTERN = "/Specs/{path:.+}.podspec.json";

  public static final String POD_PATTERN =
      "/pods/{" + PACKAGE_NAME_KEY + "}/{" + PACKAGE_VERSION_KEY + "}/{" + FILENAME_KEY + "}";

  private final Provider<CocoapodsSecurityFacet> securityFacet;

  private final Provider<ConfigurableViewFacet> viewFacet;

  private final Provider<HttpClientFacet> httpClientFacet;

  private final Provider<NegativeCacheFacet> negativeCacheFacet;

  private final Provider<CocoapodsProxyFacet> proxyFacet;

  private final Provider<CocoapodsContentFacet> contentFacet;

  private final Provider<LastAssetMaintenanceFacet> maintenanceFacet;

  private final Provider<SearchFacet> searchFacet;

  private final Provider<BrowseFacet> browseFacet;

  private final Provider<PurgeUnusedFacet> purgeUnusedFacet;

  private TimingHandler timingHandler;

  private SecurityHandler securityHandler;

  private RoutingRuleHandler routingHandler;

  private ExceptionHandler exceptionHandler;

  private HandlerContributor handlerContributor;

  private NegativeCacheHandler negativeCacheHandler;

  private ConditionalRequestHandler conditionalRequestHandler;

  private PartialFetchHandler partialFetchHandler;

  private ContentHeadersHandler contentHeadersHandler;

  private LastDownloadedHandler lastDownloadedHandler;

  private ProxyHandler proxyHandler;

  @Autowired
  public CocoapodsProxyRecipe(
      @Qualifier(ProxyType.NAME) final Type type,
      @Qualifier(CocoapodsFormat.NAME) final Format format,
      final Provider<CocoapodsSecurityFacet> securityFacet,
      final Provider<ConfigurableViewFacet> viewFacet,
      final Provider<HttpClientFacet> httpClientFacet,
      final Provider<NegativeCacheFacet> negativeCacheFacet,
      final Provider<CocoapodsProxyFacet> proxyFacet,
      final Provider<CocoapodsContentFacet> contentFacet,
      final Provider<LastAssetMaintenanceFacet> maintenanceFacet,
      final Provider<SearchFacet> searchFacet,
      final Provider<BrowseFacet> browseFacet,
      final Provider<PurgeUnusedFacet> purgeUnusedFacet)
  {
    super(type, format);
    this.securityFacet = checkNotNull(securityFacet);
    this.viewFacet = checkNotNull(viewFacet);
    this.httpClientFacet = checkNotNull(httpClientFacet);
    this.negativeCacheFacet = checkNotNull(negativeCacheFacet);
    this.proxyFacet = checkNotNull(proxyFacet);
    this.contentFacet = checkNotNull(contentFacet);
    this.maintenanceFacet = checkNotNull(maintenanceFacet);
    this.searchFacet = checkNotNull(searchFacet);
    this.browseFacet = checkNotNull(browseFacet);
    this.purgeUnusedFacet = checkNotNull(purgeUnusedFacet);
  }

  @Autowired
  public final void setHandlers(
      final TimingHandler timingHandler,
      final SecurityHandler securityHandler,
      final RoutingRuleHandler routingHandler,
      final ExceptionHandler exceptionHandler,
      final HandlerContributor handlerContributor,
      final NegativeCacheHandler negativeCacheHandler,
      final ConditionalRequestHandler conditionalRequestHandler,
      final PartialFetchHandler partialFetchHandler,
      final ContentHeadersHandler contentHeadersHandler,
      final LastDownloadedHandler lastDownloadedHandler,
      final ProxyHandler proxyHandler)
  {
    this.timingHandler = checkNotNull(timingHandler);
    this.securityHandler = checkNotNull(securityHandler);
    this.routingHandler = checkNotNull(routingHandler);
    this.exceptionHandler = checkNotNull(exceptionHandler);
    this.handlerContributor = checkNotNull(handlerContributor);
    this.negativeCacheHandler = checkNotNull(negativeCacheHandler);
    this.conditionalRequestHandler = checkNotNull(conditionalRequestHandler);
    this.partialFetchHandler = checkNotNull(partialFetchHandler);
    this.contentHeadersHandler = checkNotNull(contentHeadersHandler);
    this.lastDownloadedHandler = checkNotNull(lastDownloadedHandler);
    this.proxyHandler = checkNotNull(proxyHandler);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(securityFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(httpClientFacet.get());
    repository.attach(negativeCacheFacet.get());
    repository.attach(proxyFacet.get());
    repository.attach(contentFacet.get());
    repository.attach(maintenanceFacet.get());
    repository.attach(searchFacet.get());
    repository.attach(browseFacet.get());
    repository.attach(purgeUnusedFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();

    for (String pattern : CDN_METADATA_PATTERNS) {
      builder.route(createRoute(createGetTokenMatcher(pattern), CDN_METADATA));
    }
    builder.route(createRoute(createGetTokenMatcher(SPEC_PATTERN), SPEC));
    builder.route(createRoute(createGetTokenMatcher(POD_PATTERN), POD));

    addBrowseUnsupportedRoute(builder);

    builder.defaultHandlers(notFound());
    facet.configure(builder.create());
    return facet;
  }

  private Route createRoute(final Matcher matcher, final AssetKind assetKind) {
    Route.Builder route = new Route.Builder().matcher(matcher)
        .handler(timingHandler)
        .handler(assetKindHandler(assetKind))
        .handler(securityHandler)
        .handler(routingHandler)
        .handler(exceptionHandler);
    if (assetKind == POD) {
      route.handler(handlerContributor);
    }
    return route.handler(negativeCacheHandler)
        .handler(conditionalRequestHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(lastDownloadedHandler)
        .handler(proxyHandler)
        .create();
  }

  private static Matcher createGetTokenMatcher(final String pattern) {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD), new TokenMatcher(pattern));
  }

  private static Handler assetKindHandler(final AssetKind assetKind) {
    return context -> {
      context.getAttributes().set(AssetKind.class, assetKind);
      return context.proceed();
    };
  }
}
