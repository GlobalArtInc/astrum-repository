package org.globalart.astrum.repository.composer.internal.recipe;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.cache.NegativeCacheFacet;
import org.globalart.astrum.repository.cache.NegativeCacheHandler;
import org.globalart.astrum.repository.composer.AssetKind;
import org.globalart.astrum.repository.composer.ComposerFormat;
import org.globalart.astrum.repository.composer.internal.proxy.ComposerPackageHandler;
import org.globalart.astrum.repository.composer.internal.proxy.ComposerProviderHandler;
import org.globalart.astrum.repository.composer.internal.proxy.ComposerProxyFacet;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.httpclient.HttpClientFacet;
import org.globalart.astrum.repository.proxy.ProxyHandler;
import org.globalart.astrum.repository.purge.PurgeUnusedFacet;
import org.globalart.astrum.repository.routing.RoutingRuleHandler;
import org.globalart.astrum.repository.types.ProxyType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Matcher;
import org.globalart.astrum.repository.view.Route;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;
import org.globalart.astrum.repository.view.handlers.LastDownloadedHandler;

import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(ComposerProxyRecipe.NAME)
public class ComposerProxyRecipe
    extends ComposerRecipeSupport
{
  public static final String NAME = "composer-proxy";

  private final Provider<ComposerProxyFacet> proxyFacet;

  private final Provider<NegativeCacheFacet> negativeCacheFacet;

  private final Provider<PurgeUnusedFacet> purgeUnusedFacet;

  private final Provider<HttpClientFacet> httpClientFacet;

  private final NegativeCacheHandler negativeCacheHandler;

  private final ProxyHandler proxyHandler;

  private final RoutingRuleHandler routingRuleHandler;

  private final LastDownloadedHandler lastDownloadedHandler;

  private final ComposerProviderHandler providerHandler;

  private final ComposerPackageHandler packageHandler;

  @Autowired
  public ComposerProxyRecipe(
      @Qualifier(ProxyType.NAME) final Type type,
      @Qualifier(ComposerFormat.NAME) final Format format,
      final Provider<ComposerProxyFacet> proxyFacet,
      final Provider<NegativeCacheFacet> negativeCacheFacet,
      final Provider<PurgeUnusedFacet> purgeUnusedFacet,
      final Provider<HttpClientFacet> httpClientFacet,
      final NegativeCacheHandler negativeCacheHandler,
      final ProxyHandler proxyHandler,
      final RoutingRuleHandler routingRuleHandler,
      final LastDownloadedHandler lastDownloadedHandler,
      final ComposerProviderHandler providerHandler,
      final ComposerPackageHandler packageHandler)
  {
    super(type, format);
    this.proxyFacet = checkNotNull(proxyFacet);
    this.negativeCacheFacet = checkNotNull(negativeCacheFacet);
    this.purgeUnusedFacet = checkNotNull(purgeUnusedFacet);
    this.httpClientFacet = checkNotNull(httpClientFacet);
    this.negativeCacheHandler = checkNotNull(negativeCacheHandler);
    this.proxyHandler = checkNotNull(proxyHandler);
    this.routingRuleHandler = checkNotNull(routingRuleHandler);
    this.lastDownloadedHandler = checkNotNull(lastDownloadedHandler);
    this.providerHandler = checkNotNull(providerHandler);
    this.packageHandler = checkNotNull(packageHandler);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(contentFacet.get());
    repository.attach(securityFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(httpClientFacet.get());
    repository.attach(negativeCacheFacet.get());
    repository.attach(proxyFacet.get());
    repository.attach(searchFacet.get());
    repository.attach(browseFacet.get());
    repository.attach(purgeUnusedFacet.get());
    repository.attach(maintenanceFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();

    addBrowseUnsupportedRoute(builder);

    builder.route(proxy(packagesMatcher(), AssetKind.PACKAGES, null));
    builder.route(proxy(listMatcher(), AssetKind.LIST, null));
    builder.route(proxy(providerMatcher(), AssetKind.PROVIDER, providerHandler));
    builder.route(proxy(packageMatcher(), AssetKind.PACKAGE, packageHandler));
    builder.route(proxy(zipballMatcher(), AssetKind.ZIPBALL, null));

    builder.defaultHandlers(HttpHandlers.notFound());

    facet.configure(builder.create());

    return facet;
  }

  private Route proxy(final Matcher matcher, final AssetKind assetKind, final Handler rewriteHandler) {
    Route.Builder route = route(matcher, assetKind)
        .handler(routingRuleHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor)
        .handler(negativeCacheHandler)
        .handler(conditionalRequestHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler);
    if (rewriteHandler != null) {
      route.handler(rewriteHandler);
    }
    return route.handler(lastDownloadedHandler)
        .handler(proxyHandler)
        .create();
  }
}
