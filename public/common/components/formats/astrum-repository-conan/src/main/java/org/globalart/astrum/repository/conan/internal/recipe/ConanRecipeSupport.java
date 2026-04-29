
package org.globalart.astrum.repository.conan.internal.recipe;

import java.util.function.Function;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.RecipeSupport;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.conan.ConanContentFacet;
import org.globalart.astrum.repository.conan.internal.ConanEndpoint;
import org.globalart.astrum.repository.conan.internal.common.ConanUserHandlers;
import org.globalart.astrum.repository.conan.internal.security.ConanSecurityFacet;
import org.globalart.astrum.repository.content.browse.BrowseFacet;
import org.globalart.astrum.repository.content.maintenance.LastAssetMaintenanceFacet;
import org.globalart.astrum.repository.content.search.SearchFacet;
import org.globalart.astrum.repository.http.PartialFetchHandler;
import org.globalart.astrum.repository.httpclient.HttpClientFacet;
import org.globalart.astrum.repository.security.SecurityHandler;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Handler;
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

public abstract class ConanRecipeSupport
    extends RecipeSupport
{
  protected Provider<ConanSecurityFacet> securityFacet;

  protected Provider<ConfigurableViewFacet> viewFacet;

  protected Provider<ConanContentFacet> contentFacet;

  protected Provider<HttpClientFacet> httpClientFacet;

  protected Provider<LastAssetMaintenanceFacet> maintenanceFacet;

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

  protected ConanUserHandlers userHandlers;

  protected ConanRecipeSupport(final Type type, final Format format) {
    super(type, format);
  }

  @Autowired
  public final void setDependencies(
      final Provider<ConanSecurityFacet> securityFacet,
      final Provider<ConfigurableViewFacet> viewFacet,
      final Provider<ConanContentFacet> contentFacet,
      final Provider<HttpClientFacet> httpClientFacet,
      final Provider<LastAssetMaintenanceFacet> maintenanceFacet,
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
      final ConanUserHandlers userHandlers)
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
    this.userHandlers = checkNotNull(userHandlers);
  }

  protected void addUserRoutes(final Router.Builder builder, final Function<Repository, String> capabilities) {
    Handler ping = userHandlers.ping(capabilities);
    builder.route(simpleRoute(ConanEndpoint.PING_V1, ping));
    builder.route(simpleRoute(ConanEndpoint.PING_V2, ping));
    builder.route(simpleRoute(ConanEndpoint.AUTHENTICATE_V1, userHandlers.authenticate));
    builder.route(simpleRoute(ConanEndpoint.AUTHENTICATE_V2, userHandlers.authenticate));
    builder.route(simpleRoute(ConanEndpoint.CHECK_CREDENTIALS_V1, userHandlers.checkCredentials));
    builder.route(simpleRoute(ConanEndpoint.CHECK_CREDENTIALS_V2, userHandlers.checkCredentials));
  }

  protected Route simpleRoute(final ConanEndpoint endpoint, final Handler... handlers) {
    Route.Builder route = new Route.Builder().matcher(endpoint.matcher())
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor);
    for (Handler handler : handlers) {
      route.handler(handler);
    }
    return route.create();
  }

  protected static boolean isUserEndpoint(final ConanEndpoint endpoint) {
    return endpoint.protocol() == 0;
  }

  protected static boolean isFileDownload(final ConanEndpoint endpoint) {
    return switch (endpoint) {
      case RECIPE_FILE, PACKAGE_FILE, V1_RECIPE_FILE, V1_PACKAGE_FILE -> true;
      default -> false;
    };
  }

  protected static boolean isSearch(final ConanEndpoint endpoint) {
    return endpoint == ConanEndpoint.SEARCH_V1 || endpoint == ConanEndpoint.SEARCH_V2;
  }
}
