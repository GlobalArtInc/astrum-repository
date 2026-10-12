package org.globalart.astrum.repository.docker.internal.recipe;

import java.util.List;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.docker.DockerFormat;
import org.globalart.astrum.repository.docker.internal.group.DockerGroupHandler;
import org.globalart.astrum.repository.docker.internal.v2.DockerRoute;
import org.globalart.astrum.repository.docker.internal.v2.DockerRouteMatcher;
import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;

import jakarta.inject.Provider;
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
@Qualifier(DockerGroupRecipe.NAME)
public class DockerGroupRecipe
    extends DockerRecipeSupport
{
  public static final String NAME = "docker-group";

  private final Provider<GroupFacet> groupFacet;

  private final DockerGroupHandler groupHandler;

  @Autowired
  public DockerGroupRecipe(
      @Qualifier(GroupType.NAME) final Type type,
      @Qualifier(DockerFormat.NAME) final Format format,
      @Qualifier("default") final Provider<GroupFacet> groupFacet,
      final DockerGroupHandler groupHandler)
  {
    super(type, format);
    this.groupFacet = checkNotNull(groupFacet);
    this.groupHandler = checkNotNull(groupHandler);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(contentFacet.get());
    repository.attach(securityFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(groupFacet.get());
    repository.attach(browseFacet.get());
    repository.attach(connectorFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();

    metadataRoutes(builder, groupHandler, groupHandler, groupHandler);
    builder.route(route(DockerRoute.Type.MANIFEST, DockerRouteMatcher.MANIFEST, List.of(GET, HEAD, PUT, DELETE),
        groupHandler));
    builder.route(route(DockerRoute.Type.BLOB, DockerRouteMatcher.BLOB, List.of(GET, HEAD, DELETE),
        partialFetchHandler, groupHandler));
    builder.route(route(DockerRoute.Type.UPLOADS, DockerRouteMatcher.UPLOADS, List.of(POST), groupHandler));
    builder.route(route(DockerRoute.Type.UPLOAD, DockerRouteMatcher.UPLOAD, List.of(GET, PATCH, PUT, DELETE),
        groupHandler));
    builder.route(unsupportedRoute());

    addBrowseUnsupportedRoute(builder);

    builder.defaultHandlers(HttpHandlers.notFound());

    facet.configure(builder.create());
    return facet;
  }
}
