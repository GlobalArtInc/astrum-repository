package org.globalart.astrum.repository.helm.internal.content.recipe;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.cache.NegativeCacheFacet;
import org.globalart.astrum.repository.cache.NegativeCacheHandler;
import org.globalart.astrum.repository.helm.internal.HelmFormat;
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
import static org.globalart.astrum.repository.http.HttpHandlers.notFound;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(HelmProxyRecipe.NAME)
public class HelmProxyRecipe
    extends HelmRecipeSupport
{
  public static final String NAME = "helm-proxy";

  private final Provider<NegativeCacheFacet> negativeCacheFacet;

  private final Provider<HelmProxyFacet> proxyFacet;

  private final Provider<PurgeUnusedFacet> purgeUnusedFacet;

  private final NegativeCacheHandler negativeCacheHandler;

  private final ProxyHandler proxyHandler;

  private final RoutingRuleHandler routingRuleHandler;

  @Autowired
  public HelmProxyRecipe(
      @Qualifier(ProxyType.NAME) final Type type,
      @Qualifier(HelmFormat.NAME) final Format format,
      final Provider<NegativeCacheFacet> negativeCacheFacet,
      final Provider<HelmProxyFacet> proxyFacet,
      final Provider<PurgeUnusedFacet> purgeUnusedFacet,
      final NegativeCacheHandler negativeCacheHandler,
      final ProxyHandler proxyHandler,
      final RoutingRuleHandler routingRuleHandler)
  {
    super(type, format);
    this.negativeCacheFacet = checkNotNull(negativeCacheFacet);
    this.proxyFacet = checkNotNull(proxyFacet);
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
    repository.attach(maintenanceFacet.get());
    repository.attach(negativeCacheFacet.get());
    repository.attach(proxyFacet.get());
    repository.attach(contentFacet.get());
    repository.attach(searchFacet.get());
    repository.attach(browseFacet.get());
    repository.attach(purgeUnusedFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();
    addBrowseUnsupportedRoute(builder);
    for (Matcher matcher : new Matcher[]{indexMatcher(), packageMatcher()}) {
      builder.route(new Route.Builder().matcher(matcher)
          .handler(timingHandler)
          .handler(securityHandler)
          .handler(routingRuleHandler)
          .handler(exceptionHandler)
          .handler(handlerContributor)
          .handler(negativeCacheHandler)
          .handler(conditionalRequestHandler)
          .handler(partialFetchHandler)
          .handler(contentHeadersHandler)
          .handler(proxyHandler)
          .create());
    }
    builder.defaultHandlers(notFound());
    facet.configure(builder.create());
    return facet;
  }
}
