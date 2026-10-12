package org.globalart.astrum.repository.docker.internal.recipe;

import java.util.List;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.docker.DockerFormat;
import org.globalart.astrum.repository.docker.internal.v2.DockerHostedHandler;
import org.globalart.astrum.repository.docker.internal.v2.DockerRoute;
import org.globalart.astrum.repository.docker.internal.v2.DockerRouteMatcher;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.http.HttpMethods.DELETE;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;
import static org.globalart.astrum.repository.http.HttpMethods.PATCH;
import static org.globalart.astrum.repository.http.HttpMethods.POST;
import static org.globalart.astrum.repository.http.HttpMethods.PUT;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(DockerHostedRecipe.NAME)
public class DockerHostedRecipe
    extends DockerRecipeSupport
{
  public static final String NAME = "docker-hosted";

  private final DockerHostedHandler hostedHandler;

  @Autowired
  public DockerHostedRecipe(
      @Qualifier(HostedType.NAME) final Type type,
      @Qualifier(DockerFormat.NAME) final Format format,
      final DockerHostedHandler hostedHandler)
  {
    super(type, format);
    this.hostedHandler = checkNotNull(hostedHandler);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(contentFacet.get());
    repository.attach(securityFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(maintenanceFacet.get());
    repository.attach(searchFacet.get());
    repository.attach(browseFacet.get());
    repository.attach(connectorFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();

    metadataRoutes(builder, metadataHandler, metadataHandler, metadataHandler);
    builder.route(route(DockerRoute.Type.MANIFEST, DockerRouteMatcher.MANIFEST, List.of(GET, HEAD),
        lastDownloadedHandler, hostedHandler));
    builder.route(route(DockerRoute.Type.MANIFEST, DockerRouteMatcher.MANIFEST, List.of(PUT, DELETE),
        hostedHandler));
    builder.route(route(DockerRoute.Type.BLOB, DockerRouteMatcher.BLOB, List.of(GET, HEAD),
        partialFetchHandler, lastDownloadedHandler, hostedHandler));
    builder.route(route(DockerRoute.Type.BLOB, DockerRouteMatcher.BLOB, List.of(DELETE), hostedHandler));
    builder.route(route(DockerRoute.Type.UPLOADS, DockerRouteMatcher.UPLOADS, List.of(POST), hostedHandler));
    builder.route(route(DockerRoute.Type.UPLOAD, DockerRouteMatcher.UPLOAD, List.of(GET, PATCH, PUT, DELETE),
        hostedHandler));
    builder.route(unsupportedRoute());

    addBrowseUnsupportedRoute(builder);

    builder.defaultHandlers(HttpHandlers.notFound());

    facet.configure(builder.create());
    return facet;
  }
}
