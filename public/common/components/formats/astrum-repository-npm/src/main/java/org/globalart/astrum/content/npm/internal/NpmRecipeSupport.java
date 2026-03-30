package org.globalart.astrum.content.npm.internal;

import org.globalart.astrum.content.npm.NpmContentFacet;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.RecipeSupport;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.content.browse.BrowseFacet;
import org.globalart.astrum.repository.content.search.SearchFacet;
import org.globalart.astrum.repository.http.PartialFetchHandler;
import org.globalart.astrum.repository.npm.internal.NpmHandlers;
import org.globalart.astrum.repository.npm.internal.NpmPingHandler;
import org.globalart.astrum.repository.npm.internal.NpmSecurityFacet;
import org.globalart.astrum.repository.npm.internal.NpmTokenFacet;
import org.globalart.astrum.repository.npm.internal.NpmWhoamiHandler;
import org.globalart.astrum.repository.routing.RoutingRuleHandler;
import org.globalart.astrum.repository.security.SecurityHandler;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
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
import static org.globalart.astrum.repository.http.HttpMethods.DELETE;
import static org.globalart.astrum.repository.http.HttpMethods.PUT;
import static org.globalart.astrum.repository.npm.internal.NpmPaths.pingMatcher;
import static org.globalart.astrum.repository.npm.internal.NpmPaths.tokenMatcher;
import static org.globalart.astrum.repository.npm.internal.NpmPaths.userMatcher;
import static org.globalart.astrum.repository.npm.internal.NpmPaths.whoamiMatcher;

public abstract class NpmRecipeSupport
    extends RecipeSupport
{
  protected Provider<NpmSecurityFacet> securityFacet;

  protected Provider<ConfigurableViewFacet> viewFacet;

  protected Provider<NpmContentFacet> contentFacet;

  protected Provider<SearchFacet> searchFacet;

  protected Provider<BrowseFacet> browseFacet;

  protected Provider<NpmTokenFacet> tokenFacet;

  protected ExceptionHandler exceptionHandler;

  protected TimingHandler timingHandler;

  protected SecurityHandler securityHandler;

  protected PartialFetchHandler partialFetchHandler;

  protected ConditionalRequestHandler conditionalRequestHandler;

  protected ContentHeadersHandler contentHeadersHandler;

  protected LastDownloadedHandler lastDownloadedHandler;

  protected HandlerContributor handlerContributor;

  protected RoutingRuleHandler routingHandler;

  protected NpmWhoamiHandler npmWhoamiHandler;

  protected NpmPingHandler pingHandler;

  protected NpmRecipeSupport(final Type type, final Format format) {
    super(type, format);
  }

  @Autowired
  public final void setDependencies(
      final Provider<NpmSecurityFacet> securityFacet,
      final Provider<ConfigurableViewFacet> viewFacet,
      final Provider<NpmContentFacet> contentFacet,
      final Provider<SearchFacet> searchFacet,
      final Provider<BrowseFacet> browseFacet,
      final Provider<NpmTokenFacet> tokenFacet,
      final ExceptionHandler exceptionHandler,
      final TimingHandler timingHandler,
      final SecurityHandler securityHandler,
      final PartialFetchHandler partialFetchHandler,
      final ConditionalRequestHandler conditionalRequestHandler,
      final ContentHeadersHandler contentHeadersHandler,
      final LastDownloadedHandler lastDownloadedHandler,
      final HandlerContributor handlerContributor,
      final RoutingRuleHandler routingHandler,
      final NpmWhoamiHandler npmWhoamiHandler,
      final NpmPingHandler pingHandler)
  {
    this.securityFacet = checkNotNull(securityFacet);
    this.viewFacet = checkNotNull(viewFacet);
    this.contentFacet = checkNotNull(contentFacet);
    this.searchFacet = checkNotNull(searchFacet);
    this.browseFacet = checkNotNull(browseFacet);
    this.tokenFacet = checkNotNull(tokenFacet);
    this.exceptionHandler = checkNotNull(exceptionHandler);
    this.timingHandler = checkNotNull(timingHandler);
    this.securityHandler = checkNotNull(securityHandler);
    this.partialFetchHandler = checkNotNull(partialFetchHandler);
    this.conditionalRequestHandler = checkNotNull(conditionalRequestHandler);
    this.contentHeadersHandler = checkNotNull(contentHeadersHandler);
    this.lastDownloadedHandler = checkNotNull(lastDownloadedHandler);
    this.handlerContributor = checkNotNull(handlerContributor);
    this.routingHandler = checkNotNull(routingHandler);
    this.npmWhoamiHandler = checkNotNull(npmWhoamiHandler);
    this.pingHandler = checkNotNull(pingHandler);
  }

  protected void createCommonRoutes(final Router.Builder builder) {
    builder.route(whoamiMatcher()
        .handler(timingHandler)
        .handler(npmWhoamiHandler)
        .create());

    builder.route(pingMatcher()
        .handler(timingHandler)
        .handler(pingHandler)
        .create());

    builder.route(userMatcher(PUT)
        .handler(timingHandler)
        .handler(NpmHandlers.npmErrorHandler)
        .handler(NpmHandlers.createToken)
        .create());

    builder.route(tokenMatcher(DELETE)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(NpmHandlers.npmErrorHandler)
        .handler(NpmHandlers.deleteToken)
        .create());
  }
}
