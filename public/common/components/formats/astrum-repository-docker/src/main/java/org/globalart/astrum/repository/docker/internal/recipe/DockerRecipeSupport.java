package org.globalart.astrum.repository.docker.internal.recipe;

import java.util.List;
import java.util.regex.Pattern;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.RecipeSupport;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.content.browse.BrowseFacet;
import org.globalart.astrum.repository.content.search.SearchFacet;
import org.globalart.astrum.repository.docker.DockerContentFacet;
import org.globalart.astrum.repository.docker.internal.DockerConnectorFacet;
import org.globalart.astrum.repository.docker.internal.DockerMaintenanceFacet;
import org.globalart.astrum.repository.docker.internal.security.DockerSecurityFacet;
import org.globalart.astrum.repository.docker.internal.v2.DockerError;
import org.globalart.astrum.repository.docker.internal.v2.DockerMetadataHandler;
import org.globalart.astrum.repository.docker.internal.v2.DockerRequestHandler;
import org.globalart.astrum.repository.docker.internal.v2.DockerResponses;
import org.globalart.astrum.repository.docker.internal.v2.DockerRoute;
import org.globalart.astrum.repository.docker.internal.v2.DockerRouteMatcher;
import org.globalart.astrum.repository.http.PartialFetchHandler;
import org.globalart.astrum.repository.security.SecurityHandler;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Route;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.handlers.ExceptionHandler;
import org.globalart.astrum.repository.view.handlers.HandlerContributor;
import org.globalart.astrum.repository.view.handlers.LastDownloadedHandler;
import org.globalart.astrum.repository.view.handlers.TimingHandler;

import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;

public abstract class DockerRecipeSupport
    extends RecipeSupport
{
  protected Provider<DockerContentFacet> contentFacet;

  protected Provider<DockerSecurityFacet> securityFacet;

  protected Provider<ConfigurableViewFacet> viewFacet;

  protected Provider<SearchFacet> searchFacet;

  protected Provider<BrowseFacet> browseFacet;

  protected Provider<DockerMaintenanceFacet> maintenanceFacet;

  protected Provider<DockerConnectorFacet> connectorFacet;

  protected ExceptionHandler exceptionHandler;

  protected TimingHandler timingHandler;

  protected SecurityHandler securityHandler;

  protected PartialFetchHandler partialFetchHandler;

  protected HandlerContributor handlerContributor;

  protected LastDownloadedHandler lastDownloadedHandler;

  protected DockerRequestHandler requestHandler;

  protected DockerMetadataHandler metadataHandler;

  protected DockerRecipeSupport(final Type type, final Format format) {
    super(type, format);
  }

  @Autowired
  public final void setDependencies(
      final Provider<DockerContentFacet> contentFacet,
      final Provider<DockerSecurityFacet> securityFacet,
      final Provider<ConfigurableViewFacet> viewFacet,
      final Provider<SearchFacet> searchFacet,
      final Provider<BrowseFacet> browseFacet,
      final Provider<DockerMaintenanceFacet> maintenanceFacet,
      final Provider<DockerConnectorFacet> connectorFacet,
      final ExceptionHandler exceptionHandler,
      final TimingHandler timingHandler,
      final SecurityHandler securityHandler,
      final PartialFetchHandler partialFetchHandler,
      final HandlerContributor handlerContributor,
      final LastDownloadedHandler lastDownloadedHandler,
      final DockerRequestHandler requestHandler,
      final DockerMetadataHandler metadataHandler)
  {
    this.contentFacet = checkNotNull(contentFacet);
    this.securityFacet = checkNotNull(securityFacet);
    this.viewFacet = checkNotNull(viewFacet);
    this.searchFacet = checkNotNull(searchFacet);
    this.browseFacet = checkNotNull(browseFacet);
    this.maintenanceFacet = checkNotNull(maintenanceFacet);
    this.connectorFacet = checkNotNull(connectorFacet);
    this.exceptionHandler = checkNotNull(exceptionHandler);
    this.timingHandler = checkNotNull(timingHandler);
    this.securityHandler = checkNotNull(securityHandler);
    this.partialFetchHandler = checkNotNull(partialFetchHandler);
    this.handlerContributor = checkNotNull(handlerContributor);
    this.lastDownloadedHandler = checkNotNull(lastDownloadedHandler);
    this.requestHandler = checkNotNull(requestHandler);
    this.metadataHandler = checkNotNull(metadataHandler);
  }

  protected Route route(
      final DockerRoute.Type type,
      final Pattern pattern,
      final List<String> actions,
      final Object... handlers)
  {
    Route.Builder builder = new Route.Builder()
        .matcher(new DockerRouteMatcher(type, pattern, actions.toArray(new String[0])))
        .handler(timingHandler)
        .handler(requestHandler)
        .handler(securityHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor);
    for (Object handler : handlers) {
      if (handler instanceof Handler viewHandler) {
        builder.handler(viewHandler);
      }
      else {
        builder.handler((org.globalart.astrum.repository.recipe.Handler) handler);
      }
    }
    return builder.create();
  }

  protected void metadataRoutes(
      final Router.Builder builder,
      final Handler tagsHandler,
      final Handler catalogHandler,
      final Handler referrersHandler)
  {
    builder.route(route(DockerRoute.Type.BASE, DockerRouteMatcher.BASE, List.of(GET, HEAD), metadataHandler));
    builder.route(route(DockerRoute.Type.CATALOG, DockerRouteMatcher.CATALOG, List.of(GET), catalogHandler));
    builder.route(route(DockerRoute.Type.TAGS, DockerRouteMatcher.TAGS, List.of(GET), tagsHandler));
    builder.route(route(DockerRoute.Type.REFERRERS, DockerRouteMatcher.REFERRERS, List.of(GET), referrersHandler));
  }

  protected static List<String> readActions() {
    return List.of(GET, HEAD);
  }

  protected Route unsupportedRoute() {
    return new Route.Builder()
        .matcher(context -> context.getRequest().getPath().startsWith("/v2/"))
        .handler(timingHandler)
        .handler(requestHandler)
        .handler(context -> DockerResponses.error(DockerError.UNSUPPORTED,
            "repository " + context.getRepository().getName() + " does not support "
                + context.getRequest().getAction() + " on this endpoint"))
        .create();
  }
}
