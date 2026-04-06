package org.globalart.astrum.repository.nuget.internal.recipe;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.cache.NegativeCacheFacet;
import org.globalart.astrum.repository.cache.NegativeCacheHandler;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.httpclient.HttpClientFacet;
import org.globalart.astrum.repository.nuget.internal.NugetFormat;
import org.globalart.astrum.repository.nuget.internal.proxy.NugetProxyFacet;
import org.globalart.astrum.repository.nuget.internal.proxy.NugetProxyHandlers;
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

import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.anyRead;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.assetDownload;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.autocomplete;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.flatFile;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.flatNuspec;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.flatPackage;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.flatVersions;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.query;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.registrationIndex;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.registrationLeaf;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.registrationPage;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.serviceIndex;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.v2Download;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.v2Root;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(NugetProxyRecipe.NAME)
public class NugetProxyRecipe
    extends NugetRecipeSupport
{
  public static final String NAME = "nuget-proxy";

  private final Provider<HttpClientFacet> httpClientFacet;

  private final Provider<NegativeCacheFacet> negativeCacheFacet;

  private final Provider<NugetProxyFacet> proxyFacet;

  private final Provider<PurgeUnusedFacet> purgeUnusedFacet;

  private final NegativeCacheHandler negativeCacheHandler;

  private final ProxyHandler proxyHandler;

  private final RoutingRuleHandler routingRuleHandler;

  private final NugetProxyHandlers handlers;

  @Autowired
  public NugetProxyRecipe(
      @Qualifier(ProxyType.NAME) final Type type,
      @Qualifier(NugetFormat.NAME) final Format format,
      final Provider<HttpClientFacet> httpClientFacet,
      final Provider<NegativeCacheFacet> negativeCacheFacet,
      final Provider<NugetProxyFacet> proxyFacet,
      final Provider<PurgeUnusedFacet> purgeUnusedFacet,
      final NegativeCacheHandler negativeCacheHandler,
      final ProxyHandler proxyHandler,
      final RoutingRuleHandler routingRuleHandler,
      final NugetProxyHandlers handlers)
  {
    super(type, format);
    this.httpClientFacet = checkNotNull(httpClientFacet);
    this.negativeCacheFacet = checkNotNull(negativeCacheFacet);
    this.proxyFacet = checkNotNull(proxyFacet);
    this.purgeUnusedFacet = checkNotNull(purgeUnusedFacet);
    this.negativeCacheHandler = checkNotNull(negativeCacheHandler);
    this.proxyHandler = checkNotNull(proxyHandler);
    this.routingRuleHandler = checkNotNull(routingRuleHandler);
    this.handlers = checkNotNull(handlers);
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
    builder.route(simpleRoute(v2Root(), handlers.v2PassThrough));
    addBrowseUnsupportedRoute(builder);
    builder.route(simpleRoute(serviceIndex(), handlers.serviceIndex));
    for (Matcher matcher : new Matcher[]{flatVersions(), flatPackage(), flatNuspec(), flatFile(),
        registrationIndex(), registrationPage(), registrationLeaf()}) {
      builder.route(proxyRoute(matcher, handlers.requireV3));
    }
    builder.route(simpleRoute(query(), handlers.search));
    builder.route(simpleRoute(autocomplete(), handlers.autocomplete));
    builder.route(proxyRoute(v2Download(), handlers.requireV2));
    builder.route(proxyRoute(assetDownload(), handlers.anyVersion));
    builder.route(simpleRoute(anyRead(), handlers.v2PassThrough));
    builder.defaultHandlers(HttpHandlers.notFound());
    facet.configure(builder.create());
    return facet;
  }

  private Route proxyRoute(final Matcher matcher, final Handler guard) {
    return new Route.Builder().matcher(matcher)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(routingRuleHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor)
        .handler(guard)
        .handler(negativeCacheHandler)
        .handler(conditionalRequestHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(lastDownloadedHandler)
        .handler(proxyHandler)
        .create();
  }
}
