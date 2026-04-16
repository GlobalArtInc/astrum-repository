package org.globalart.astrum.repository.conda.internal.recipe;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.RecipeSupport;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.conda.AssetKind;
import org.globalart.astrum.repository.conda.CondaContentFacet;
import org.globalart.astrum.repository.conda.internal.CondaMaintenanceFacet;
import org.globalart.astrum.repository.conda.internal.security.CondaSecurityFacet;
import org.globalart.astrum.repository.conda.util.CondaPathUtils;
import org.globalart.astrum.repository.content.browse.BrowseFacet;
import org.globalart.astrum.repository.content.search.SearchFacet;
import org.globalart.astrum.repository.http.PartialFetchHandler;
import org.globalart.astrum.repository.routing.RoutingRuleHandler;
import org.globalart.astrum.repository.security.SecurityHandler;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Route;
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

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;

public abstract class CondaRecipeSupport
    extends RecipeSupport
{
  protected Provider<BrowseFacet> browseFacet;

  protected Provider<CondaContentFacet> contentFacet;

  protected Provider<CondaSecurityFacet> securityFacet;

  protected Provider<ConfigurableViewFacet> viewFacet;

  protected Provider<SearchFacet> searchFacet;

  protected Provider<CondaMaintenanceFacet> maintenanceFacet;

  protected ExceptionHandler exceptionHandler;

  protected TimingHandler timingHandler;

  protected RoutingRuleHandler routingHandler;

  protected SecurityHandler securityHandler;

  protected PartialFetchHandler partialFetchHandler;

  protected ConditionalRequestHandler conditionalRequestHandler;

  protected ContentHeadersHandler contentHeadersHandler;

  protected HandlerContributor handlerContributor;

  protected LastDownloadedHandler lastDownloadedHandler;

  protected CondaRecipeSupport(final Type type, final Format format) {
    super(type, format);
  }

  @Autowired
  public final void setDependencies(
      final Provider<BrowseFacet> browseFacet,
      final Provider<CondaContentFacet> contentFacet,
      final Provider<CondaSecurityFacet> securityFacet,
      final Provider<ConfigurableViewFacet> viewFacet,
      final Provider<SearchFacet> searchFacet,
      final Provider<CondaMaintenanceFacet> maintenanceFacet,
      final ExceptionHandler exceptionHandler,
      final TimingHandler timingHandler,
      final RoutingRuleHandler routingHandler,
      final SecurityHandler securityHandler,
      final PartialFetchHandler partialFetchHandler,
      final ConditionalRequestHandler conditionalRequestHandler,
      final ContentHeadersHandler contentHeadersHandler,
      final HandlerContributor handlerContributor,
      final LastDownloadedHandler lastDownloadedHandler)
  {
    this.browseFacet = checkNotNull(browseFacet);
    this.contentFacet = checkNotNull(contentFacet);
    this.securityFacet = checkNotNull(securityFacet);
    this.viewFacet = checkNotNull(viewFacet);
    this.searchFacet = checkNotNull(searchFacet);
    this.maintenanceFacet = checkNotNull(maintenanceFacet);
    this.exceptionHandler = checkNotNull(exceptionHandler);
    this.timingHandler = checkNotNull(timingHandler);
    this.routingHandler = checkNotNull(routingHandler);
    this.securityHandler = checkNotNull(securityHandler);
    this.partialFetchHandler = checkNotNull(partialFetchHandler);
    this.conditionalRequestHandler = checkNotNull(conditionalRequestHandler);
    this.contentHeadersHandler = checkNotNull(contentHeadersHandler);
    this.handlerContributor = checkNotNull(handlerContributor);
    this.lastDownloadedHandler = checkNotNull(lastDownloadedHandler);
  }

  protected Route.Builder route(final AssetKind assetKind, final String... actions) {
    return new Route.Builder()
        .matcher(LogicMatchers.and(
            new ActionMatcher(actions.length == 0 ? new String[]{GET, HEAD} : actions),
            new TokenMatcher(CondaPathUtils.pattern(assetKind))))
        .handler(timingHandler)
        .handler(assetKindHandler(assetKind))
        .handler(securityHandler);
  }

  protected static Handler assetKindHandler(final AssetKind assetKind) {
    return context -> {
      context.getAttributes().set(AssetKind.class, assetKind);
      return context.proceed();
    };
  }
}
