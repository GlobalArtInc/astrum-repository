package org.globalart.astrum.repository.conda.internal.recipe;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.cache.NegativeCacheFacet;
import org.globalart.astrum.repository.cache.NegativeCacheHandler;
import org.globalart.astrum.repository.conda.AssetKind;
import org.globalart.astrum.repository.conda.CondaFormat;
import org.globalart.astrum.repository.conda.internal.proxy.CondaProxyFacet;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.httpclient.HttpClientFacet;
import org.globalart.astrum.repository.proxy.ProxyHandler;
import org.globalart.astrum.repository.purge.PurgeUnusedFacet;
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
@Qualifier(CondaProxyRecipe.NAME)
public class CondaProxyRecipe
    extends CondaRecipeSupport
{
  public static final String NAME = "conda-proxy";

  private final Provider<CondaProxyFacet> proxyFacet;

  private final Provider<HttpClientFacet> httpClientFacet;

  private final Provider<NegativeCacheFacet> negativeCacheFacet;

  private final Provider<PurgeUnusedFacet> purgeUnusedFacet;

  private final NegativeCacheHandler negativeCacheHandler;

  private final ProxyHandler proxyHandler;

  @Autowired
  public CondaProxyRecipe(
      @Qualifier(ProxyType.NAME) final Type type,
      @Qualifier(CondaFormat.NAME) final Format format,
      final Provider<CondaProxyFacet> proxyFacet,
      final Provider<HttpClientFacet> httpClientFacet,
      final Provider<NegativeCacheFacet> negativeCacheFacet,
      final Provider<PurgeUnusedFacet> purgeUnusedFacet,
      final NegativeCacheHandler negativeCacheHandler,
      final ProxyHandler proxyHandler)
  {
    super(type, format);
    this.proxyFacet = checkNotNull(proxyFacet);
    this.httpClientFacet = checkNotNull(httpClientFacet);
    this.negativeCacheFacet = checkNotNull(negativeCacheFacet);
    this.purgeUnusedFacet = checkNotNull(purgeUnusedFacet);
    this.negativeCacheHandler = checkNotNull(negativeCacheHandler);
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
    repository.attach(searchFacet.get());
    repository.attach(purgeUnusedFacet.get());
    repository.attach(browseFacet.get());
    repository.attach(maintenanceFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();

    for (AssetKind assetKind : AssetKind.values()) {
      if (assetKind.isSubdirLevel()) {
        builder.route(proxyRoute(assetKind));
      }
    }
    for (AssetKind assetKind : AssetKind.values()) {
      if (!assetKind.isSubdirLevel()) {
        builder.route(proxyRoute(assetKind));
      }
    }

    addBrowseUnsupportedRoute(builder);

    builder.defaultHandlers(HttpHandlers.notFound());

    facet.configure(builder.create());

    return facet;
  }

  private Route proxyRoute(final AssetKind assetKind) {
    Route.Builder route = route(assetKind)
        .handler(routingHandler)
        .handler(exceptionHandler);
    if (assetKind.isPackage()) {
      route.handler(handlerContributor);
    }
    route.handler(negativeCacheHandler)
        .handler(conditionalRequestHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler);
    if (assetKind.isPackage()) {
      route.handler(lastDownloadedHandler);
    }
    return route.handler(proxyHandler).create();
  }
}
