package org.globalart.astrum.repository.pypi.internal.content.recipe;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.RecipeSupport;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.content.browse.BrowseFacet;
import org.globalart.astrum.repository.content.search.SearchFacet;
import org.globalart.astrum.repository.http.PartialFetchHandler;
import org.globalart.astrum.repository.pypi.content.PypiContentFacet;
import org.globalart.astrum.repository.pypi.internal.AssetKind;
import org.globalart.astrum.repository.pypi.internal.PyPiSecurityFacet;
import org.globalart.astrum.repository.pypi.internal.content.PyPiIndexFacetImpl;
import org.globalart.astrum.repository.pypi.internal.content.PyPiLastAssetMaintenanceFacet;
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
import static org.globalart.astrum.repository.http.HttpMethods.POST;

public abstract class PyPiRecipeSupport
    extends RecipeSupport
{
  private static final String NAME_PATTERN = "[0-9a-zA-Z._-]+";

  protected Provider<BrowseFacet> browseFacet;

  protected Provider<PypiContentFacet> contentFacet;

  protected Provider<PyPiSecurityFacet> securityFacet;

  protected Provider<ConfigurableViewFacet> viewFacet;

  protected Provider<SearchFacet> searchFacet;

  protected Provider<PyPiLastAssetMaintenanceFacet> maintenanceFacet;

  protected Provider<PyPiIndexFacetImpl> indexFacet;

  protected ExceptionHandler exceptionHandler;

  protected TimingHandler timingHandler;

  protected RoutingRuleHandler routingHandler;

  protected SecurityHandler securityHandler;

  protected PartialFetchHandler partialFetchHandler;

  protected ConditionalRequestHandler conditionalRequestHandler;

  protected ContentHeadersHandler contentHeadersHandler;

  protected HandlerContributor handlerContributor;

  protected LastDownloadedHandler lastDownloadedHandler;

  protected PyPiRecipeSupport(final Type type, final Format format) {
    super(type, format);
  }

  @Autowired
  public final void setDependencies(
      final Provider<BrowseFacet> browseFacet,
      final Provider<PypiContentFacet> contentFacet,
      final Provider<PyPiSecurityFacet> securityFacet,
      final Provider<ConfigurableViewFacet> viewFacet,
      final Provider<SearchFacet> searchFacet,
      final Provider<PyPiLastAssetMaintenanceFacet> maintenanceFacet,
      final Provider<PyPiIndexFacetImpl> indexFacet,
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
    this.indexFacet = checkNotNull(indexFacet);
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

  protected static Handler assetKindHandler(final AssetKind assetKind) {
    return context -> {
      context.getAttributes().set(AssetKind.class, assetKind);
      return context.proceed();
    };
  }

  protected static Route.Builder indexMatcher() {
    return new Route.Builder().matcher(
        LogicMatchers.and(
            new ActionMatcher(GET, HEAD),
            LogicMatchers.or(
                new TokenMatcher("/simple/{name:" + NAME_PATTERN + "}"),
                new TokenMatcher("/simple/{name:" + NAME_PATTERN + "}/"))));
  }

  protected static Route.Builder rootIndexMatcher() {
    return new Route.Builder().matcher(
        LogicMatchers.and(
            new ActionMatcher(GET, HEAD),
            new TokenMatcher("/simple/")));
  }

  protected static Route.Builder packagesMatcher() {
    return new Route.Builder().matcher(
        LogicMatchers.and(
            new ActionMatcher(GET, HEAD),
            new TokenMatcher("{path:/packages/[^/].+/[^/]+/[^/]+}")));
  }

  protected static Route.Builder uploadMatcher() {
    return new Route.Builder().matcher(
        LogicMatchers.and(
            new ActionMatcher(POST),
            new TokenMatcher("/")));
  }
}
