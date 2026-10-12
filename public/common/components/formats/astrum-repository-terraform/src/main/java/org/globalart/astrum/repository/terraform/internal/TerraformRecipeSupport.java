package org.globalart.astrum.repository.terraform.internal;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.RecipeSupport;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.content.browse.BrowseFacet;
import org.globalart.astrum.repository.content.search.SearchFacet;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.http.PartialFetchHandler;
import org.globalart.astrum.repository.security.SecurityHandler;
import org.globalart.astrum.repository.terraform.TerraformContentFacet;
import org.globalart.astrum.repository.terraform.TerraformPaths;
import org.globalart.astrum.repository.terraform.internal.security.TerraformSecurityFacet;
import org.globalart.astrum.repository.terraform.internal.security.TerraformTokenHandler;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Matcher;
import org.globalart.astrum.repository.view.Route;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.handlers.ConditionalRequestHandler;
import org.globalart.astrum.repository.view.handlers.ContentHeadersHandler;
import org.globalart.astrum.repository.view.handlers.ExceptionHandler;
import org.globalart.astrum.repository.view.handlers.HandlerContributor;
import org.globalart.astrum.repository.view.handlers.LastDownloadedHandler;
import org.globalart.astrum.repository.view.handlers.TimingHandler;

import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;

import static com.google.common.base.Preconditions.checkNotNull;

public abstract class TerraformRecipeSupport
    extends RecipeSupport
{
  protected Provider<TerraformSecurityFacet> securityFacet;

  protected Provider<ConfigurableViewFacet> viewFacet;

  protected Provider<TerraformContentFacet> contentFacet;

  protected Provider<SearchFacet> searchFacet;

  protected Provider<BrowseFacet> browseFacet;

  protected TimingHandler timingHandler;

  protected SecurityHandler securityHandler;

  protected ExceptionHandler exceptionHandler;

  protected HandlerContributor handlerContributor;

  protected ConditionalRequestHandler conditionalRequestHandler;

  protected PartialFetchHandler partialFetchHandler;

  protected ContentHeadersHandler contentHeadersHandler;

  protected LastDownloadedHandler lastDownloadedHandler;

  protected TerraformDiscoveryHandler discoveryHandler;

  protected TerraformTokenHandler tokenHandler;

  protected TerraformRecipeSupport(final Type type, final Format format) {
    super(type, format);
  }

  @Autowired
  public final void setDependencies(
      final Provider<TerraformSecurityFacet> securityFacet,
      final Provider<ConfigurableViewFacet> viewFacet,
      final Provider<TerraformContentFacet> contentFacet,
      final Provider<SearchFacet> searchFacet,
      final Provider<BrowseFacet> browseFacet,
      final TimingHandler timingHandler,
      final SecurityHandler securityHandler,
      final ExceptionHandler exceptionHandler,
      final HandlerContributor handlerContributor,
      final ConditionalRequestHandler conditionalRequestHandler,
      final PartialFetchHandler partialFetchHandler,
      final ContentHeadersHandler contentHeadersHandler,
      final LastDownloadedHandler lastDownloadedHandler,
      final TerraformDiscoveryHandler discoveryHandler,
      final TerraformTokenHandler tokenHandler)
  {
    this.securityFacet = checkNotNull(securityFacet);
    this.viewFacet = checkNotNull(viewFacet);
    this.contentFacet = checkNotNull(contentFacet);
    this.searchFacet = checkNotNull(searchFacet);
    this.browseFacet = checkNotNull(browseFacet);
    this.timingHandler = checkNotNull(timingHandler);
    this.securityHandler = checkNotNull(securityHandler);
    this.exceptionHandler = checkNotNull(exceptionHandler);
    this.handlerContributor = checkNotNull(handlerContributor);
    this.conditionalRequestHandler = checkNotNull(conditionalRequestHandler);
    this.partialFetchHandler = checkNotNull(partialFetchHandler);
    this.contentHeadersHandler = checkNotNull(contentHeadersHandler);
    this.lastDownloadedHandler = checkNotNull(lastDownloadedHandler);
    this.discoveryHandler = checkNotNull(discoveryHandler);
    this.tokenHandler = checkNotNull(tokenHandler);
  }

  protected void addCommonRoutes(final Router.Builder builder) {
    builder.route(apiRoute(TerraformPaths.discovery(), discoveryHandler));
    builder.route(apiRoute(TerraformPaths.token(), tokenHandler));
  }

  protected void finish(final Router.Builder builder, final ConfigurableViewFacet facet) {
    addBrowseUnsupportedRoute(builder);
    builder.defaultHandlers(HttpHandlers.notFound());
    facet.configure(builder.create());
  }

  /**
   * A route answering with a generated document.
   */
  protected Route apiRoute(final Matcher matcher, final Handler... handlers) {
    Route.Builder route = new Route.Builder().matcher(matcher)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor);
    for (Handler handler : handlers) {
      route.handler(handler);
    }
    return route.create();
  }

  /**
   * A route answering with stored content.
   */
  protected Route contentRoute(final Matcher matcher, final Handler... handlers) {
    Route.Builder route = new Route.Builder().matcher(matcher)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor)
        .handler(conditionalRequestHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler);
    for (Handler handler : handlers) {
      route.handler(handler);
    }
    return route.create();
  }
}
