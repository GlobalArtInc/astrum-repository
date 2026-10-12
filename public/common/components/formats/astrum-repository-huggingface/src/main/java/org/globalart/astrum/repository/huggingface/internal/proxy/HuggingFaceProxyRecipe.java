package org.globalart.astrum.repository.huggingface.internal.proxy;

import java.util.Optional;
import java.util.Set;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.RecipeSupport;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.cache.NegativeCacheFacet;
import org.globalart.astrum.repository.content.browse.BrowseFacet;
import org.globalart.astrum.repository.content.maintenance.LastAssetMaintenanceFacet;
import org.globalart.astrum.repository.content.search.SearchFacet;
import org.globalart.astrum.repository.http.PartialFetchHandler;
import org.globalart.astrum.repository.httpclient.HttpClientFacet;
import org.globalart.astrum.repository.huggingface.HuggingFaceFormat;
import org.globalart.astrum.repository.huggingface.internal.HuggingFaceContentFacet;
import org.globalart.astrum.repository.huggingface.internal.HuggingFacePaths;
import org.globalart.astrum.repository.huggingface.internal.HuggingFaceRequest;
import org.globalart.astrum.repository.huggingface.internal.HuggingFaceRequest.Kind;
import org.globalart.astrum.repository.huggingface.internal.security.HuggingFaceSecurityFacet;
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

import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.http.HttpHandlers.notFound;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;
import static org.globalart.astrum.repository.http.HttpMethods.POST;

/**
 * Proxy of the Hugging Face Hub for {@code huggingface_hub} and every tool that honours {@code HF_ENDPOINT}.
 * Available from the database version that introduced Hugging Face proxy repositories.
 */
@AvailabilityVersion(from = "2.12")
@Component
@Qualifier(HuggingFaceProxyRecipe.NAME)
public class HuggingFaceProxyRecipe
    extends RecipeSupport
{
  public static final String NAME = "huggingface-proxy";

  private final Provider<HuggingFaceSecurityFacet> securityFacet;

  private final Provider<ConfigurableViewFacet> viewFacet;

  private final Provider<HttpClientFacet> httpClientFacet;

  private final Provider<NegativeCacheFacet> negativeCacheFacet;

  private final Provider<LastAssetMaintenanceFacet> maintenanceFacet;

  private final Provider<HuggingFaceProxyFacet> proxyFacet;

  private final Provider<HuggingFaceContentFacet> contentFacet;

  private final Provider<SearchFacet> searchFacet;

  private final Provider<BrowseFacet> browseFacet;

  private final Provider<PurgeUnusedFacet> purgeUnusedFacet;

  private final TimingHandler timingHandler;

  private final SecurityHandler securityHandler;

  private final RoutingRuleHandler routingRuleHandler;

  private final ExceptionHandler exceptionHandler;

  private final HandlerContributor handlerContributor;

  private final ConditionalRequestHandler conditionalRequestHandler;

  private final PartialFetchHandler partialFetchHandler;

  private final ContentHeadersHandler contentHeadersHandler;

  private final LastDownloadedHandler lastDownloadedHandler;

  private final HuggingFaceProxyHandler proxyHandler;

  @Autowired
  public HuggingFaceProxyRecipe(
      @Qualifier(ProxyType.NAME) final Type type,
      @Qualifier(HuggingFaceFormat.NAME) final Format format,
      final Provider<HuggingFaceSecurityFacet> securityFacet,
      final Provider<ConfigurableViewFacet> viewFacet,
      final Provider<HttpClientFacet> httpClientFacet,
      final Provider<NegativeCacheFacet> negativeCacheFacet,
      final Provider<LastAssetMaintenanceFacet> maintenanceFacet,
      final Provider<HuggingFaceProxyFacet> proxyFacet,
      final Provider<HuggingFaceContentFacet> contentFacet,
      final Provider<SearchFacet> searchFacet,
      final Provider<BrowseFacet> browseFacet,
      final Provider<PurgeUnusedFacet> purgeUnusedFacet,
      final TimingHandler timingHandler,
      final SecurityHandler securityHandler,
      final RoutingRuleHandler routingRuleHandler,
      final ExceptionHandler exceptionHandler,
      final HandlerContributor handlerContributor,
      final ConditionalRequestHandler conditionalRequestHandler,
      final PartialFetchHandler partialFetchHandler,
      final ContentHeadersHandler contentHeadersHandler,
      final LastDownloadedHandler lastDownloadedHandler,
      final HuggingFaceProxyHandler proxyHandler)
  {
    super(type, format);
    this.securityFacet = checkNotNull(securityFacet);
    this.viewFacet = checkNotNull(viewFacet);
    this.httpClientFacet = checkNotNull(httpClientFacet);
    this.negativeCacheFacet = checkNotNull(negativeCacheFacet);
    this.maintenanceFacet = checkNotNull(maintenanceFacet);
    this.proxyFacet = checkNotNull(proxyFacet);
    this.contentFacet = checkNotNull(contentFacet);
    this.searchFacet = checkNotNull(searchFacet);
    this.browseFacet = checkNotNull(browseFacet);
    this.purgeUnusedFacet = checkNotNull(purgeUnusedFacet);
    this.timingHandler = checkNotNull(timingHandler);
    this.securityHandler = checkNotNull(securityHandler);
    this.routingRuleHandler = checkNotNull(routingRuleHandler);
    this.exceptionHandler = checkNotNull(exceptionHandler);
    this.handlerContributor = checkNotNull(handlerContributor);
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
    repository.attach(maintenanceFacet.get());
    repository.attach(proxyFacet.get());
    repository.attach(contentFacet.get());
    repository.attach(searchFacet.get());
    repository.attach(browseFacet.get());
    repository.attach(purgeUnusedFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();
    addBrowseUnsupportedRoute(builder);
    builder.route(new Route.Builder()
        .matcher(matcher(Set.of(GET, HEAD), Set.of(Kind.INFO, Kind.TREE)))
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(routingRuleHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor)
        .handler(proxyHandler)
        .create());
    builder.route(new Route.Builder()
        .matcher(matcher(Set.of(POST), Set.of(Kind.PATHS_INFO)))
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(routingRuleHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor)
        .handler(proxyHandler)
        .create());
    builder.route(new Route.Builder()
        .matcher(matcher(Set.of(GET, HEAD), Set.of(Kind.FILE, Kind.STORED)))
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(routingRuleHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor)
        .handler(conditionalRequestHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(lastDownloadedHandler)
        .handler(proxyHandler)
        .create());
    builder.defaultHandlers(notFound());
    facet.configure(builder.create());
    return facet;
  }

  static Matcher matcher(final Set<String> methods, final Set<Kind> kinds) {
    return context -> {
      if (!methods.contains(context.getRequest().getAction())) {
        return false;
      }
      Optional<HuggingFaceRequest> request = HuggingFacePaths.parse(context.getRequest().getPath());
      if (request.isEmpty() || !kinds.contains(request.get().kind())) {
        return false;
      }
      context.getAttributes().set(HuggingFaceRequest.class, request.get());
      return true;
    };
  }
}
