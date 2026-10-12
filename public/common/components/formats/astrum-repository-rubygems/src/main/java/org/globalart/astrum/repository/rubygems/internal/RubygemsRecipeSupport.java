package org.globalart.astrum.repository.rubygems.internal;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.RecipeSupport;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.content.browse.BrowseFacet;
import org.globalart.astrum.repository.content.search.SearchFacet;
import org.globalart.astrum.repository.http.PartialFetchHandler;
import org.globalart.astrum.repository.httpclient.HttpClientFacet;
import org.globalart.astrum.repository.rubygems.internal.security.RubygemsApiKeyHandler;
import org.globalart.astrum.repository.rubygems.internal.security.RubygemsSecurityFacet;
import org.globalart.astrum.repository.security.SecurityHandler;
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

public abstract class RubygemsRecipeSupport
    extends RecipeSupport
{
  protected Provider<RubygemsSecurityFacet> securityFacet;

  protected Provider<ConfigurableViewFacet> viewFacet;

  protected Provider<RubygemsContentFacet> contentFacet;

  protected Provider<HttpClientFacet> httpClientFacet;

  protected Provider<RubygemsMaintenanceFacet> maintenanceFacet;

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

  protected RubygemsApiErrorHandler apiErrorHandler;

  protected RubygemsIndexEtagHandler indexEtagHandler;

  protected RubygemsDependenciesHandler dependenciesHandler;

  protected RubygemsApiKeyHandler apiKeyHandler;

  protected RubygemsRecipeSupport(final Type type, final Format format) {
    super(type, format);
  }

  @Autowired
  public final void setDependencies(
      final Provider<RubygemsSecurityFacet> securityFacet,
      final Provider<ConfigurableViewFacet> viewFacet,
      final Provider<RubygemsContentFacet> contentFacet,
      final Provider<HttpClientFacet> httpClientFacet,
      final Provider<RubygemsMaintenanceFacet> maintenanceFacet,
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
      final RubygemsApiErrorHandler apiErrorHandler,
      final RubygemsIndexEtagHandler indexEtagHandler,
      final RubygemsDependenciesHandler dependenciesHandler,
      final RubygemsApiKeyHandler apiKeyHandler)
  {
    this.securityFacet = checkNotNull(securityFacet);
    this.viewFacet = checkNotNull(viewFacet);
    this.contentFacet = checkNotNull(contentFacet);
    this.httpClientFacet = checkNotNull(httpClientFacet);
    this.maintenanceFacet = checkNotNull(maintenanceFacet);
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
    this.apiErrorHandler = checkNotNull(apiErrorHandler);
    this.indexEtagHandler = checkNotNull(indexEtagHandler);
    this.dependenciesHandler = checkNotNull(dependenciesHandler);
    this.apiKeyHandler = checkNotNull(apiKeyHandler);
  }

  protected void addCommonRoutes(final Router.Builder builder) {
    builder.route(apiRoute(RubygemsPaths.apiKey(), apiKeyHandler));
    builder.route(apiRoute(RubygemsPaths.dependencies(), dependenciesHandler));
  }

  protected void addReadOnlyRoutes(final Router.Builder builder) {
    Handler readOnly = context -> RubygemsResponses.text(405, "Repository " + context.getRepository().getName()
        + " is a " + context.getRepository().getType().getValue()
        + " repository; push gems to a rubygems hosted repository instead");
    for (Matcher matcher : new Matcher[]{RubygemsPaths.push(), RubygemsPaths.yank()}) {
      builder.route(apiRoute(matcher, readOnly));
    }
  }

  protected Route indexRoute(final Matcher matcher, final Handler... handlers) {
    Route.Builder route = new Route.Builder().matcher(matcher)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor)
        .handler(conditionalRequestHandler)
        .handler(contentHeadersHandler)
        .handler(indexEtagHandler);
    for (Handler handler : handlers) {
      route.handler(handler);
    }
    return route.create();
  }

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

  protected Route apiRoute(final Matcher matcher, final Handler... handlers) {
    Route.Builder route = new Route.Builder().matcher(matcher)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(exceptionHandler)
        .handler(apiErrorHandler)
        .handler(handlerContributor);
    for (Handler handler : handlers) {
      route.handler(handler);
    }
    return route.create();
  }
}
