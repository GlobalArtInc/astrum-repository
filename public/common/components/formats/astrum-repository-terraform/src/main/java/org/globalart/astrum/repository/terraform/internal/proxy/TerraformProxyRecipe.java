package org.globalart.astrum.repository.terraform.internal.proxy;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.cache.NegativeCacheFacet;
import org.globalart.astrum.repository.cache.NegativeCacheHandler;
import org.globalart.astrum.repository.content.maintenance.LastAssetMaintenanceFacet;
import org.globalart.astrum.repository.httpclient.HttpClientFacet;
import org.globalart.astrum.repository.proxy.ProxyHandler;
import org.globalart.astrum.repository.purge.PurgeUnusedFacet;
import org.globalart.astrum.repository.routing.RoutingRuleHandler;
import org.globalart.astrum.repository.terraform.TerraformFormat;
import org.globalart.astrum.repository.terraform.TerraformPaths;
import org.globalart.astrum.repository.terraform.internal.TerraformRecipeSupport;
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
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(TerraformProxyRecipe.NAME)
public class TerraformProxyRecipe
    extends TerraformRecipeSupport
{
  public static final String NAME = "terraform-proxy";

  private final Provider<TerraformProxyFacet> proxyFacet;

  private final Provider<HttpClientFacet> httpClientFacet;

  private final Provider<NegativeCacheFacet> negativeCacheFacet;

  private final Provider<PurgeUnusedFacet> purgeUnusedFacet;

  private final Provider<LastAssetMaintenanceFacet> maintenanceFacet;

  private final NegativeCacheHandler negativeCacheHandler;

  private final RoutingRuleHandler routingRuleHandler;

  private final ProxyHandler proxyHandler;

  private final TerraformProxyHandlers handlers;

  @Autowired
  public TerraformProxyRecipe(
      @Qualifier(ProxyType.NAME) final Type type,
      @Qualifier(TerraformFormat.NAME) final Format format,
      final Provider<TerraformProxyFacet> proxyFacet,
      final Provider<HttpClientFacet> httpClientFacet,
      final Provider<NegativeCacheFacet> negativeCacheFacet,
      final Provider<PurgeUnusedFacet> purgeUnusedFacet,
      final Provider<LastAssetMaintenanceFacet> maintenanceFacet,
      final NegativeCacheHandler negativeCacheHandler,
      final RoutingRuleHandler routingRuleHandler,
      final ProxyHandler proxyHandler,
      final TerraformProxyHandlers handlers)
  {
    super(type, format);
    this.proxyFacet = checkNotNull(proxyFacet);
    this.httpClientFacet = checkNotNull(httpClientFacet);
    this.negativeCacheFacet = checkNotNull(negativeCacheFacet);
    this.purgeUnusedFacet = checkNotNull(purgeUnusedFacet);
    this.maintenanceFacet = checkNotNull(maintenanceFacet);
    this.negativeCacheHandler = checkNotNull(negativeCacheHandler);
    this.routingRuleHandler = checkNotNull(routingRuleHandler);
    this.proxyHandler = checkNotNull(proxyHandler);
    this.handlers = checkNotNull(handlers);
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
    addCommonRoutes(builder);

    builder.route(proxy(TerraformPaths.moduleVersions(), false));
    builder.route(remote(TerraformPaths.moduleDownload(), handlers.moduleDownload));
    builder.route(proxy(TerraformPaths.moduleArchive(GET, HEAD), true));

    builder.route(proxy(TerraformPaths.providerVersions(), false));
    builder.route(remote(TerraformPaths.providerDownload(), handlers.providerDownload));
    builder.route(proxy(TerraformPaths.providerFile(GET, HEAD), true));

    builder.route(remote(TerraformPaths.mirrorIndex(), handlers.mirrorIndex));
    builder.route(remote(TerraformPaths.mirrorVersion(), handlers.mirrorVersion));

    finish(builder, facet);
    return facet;
  }

  /**
   * A route served from the cache, fetching from the upstream registry when needed.
   */
  private Route proxy(final Matcher matcher, final boolean packages) {
    Route.Builder route = new Route.Builder().matcher(matcher)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(routingRuleHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor)
        .handler(negativeCacheHandler)
        .handler(conditionalRequestHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler);
    if (packages) {
      route.handler(lastDownloadedHandler);
    }
    return route.handler(proxyHandler).create();
  }

  /**
   * A route answering with a document built from upstream documents.
   */
  private Route remote(final Matcher matcher, final Handler handler) {
    return new Route.Builder().matcher(matcher)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(routingRuleHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor)
        .handler(handler)
        .create();
  }
}
