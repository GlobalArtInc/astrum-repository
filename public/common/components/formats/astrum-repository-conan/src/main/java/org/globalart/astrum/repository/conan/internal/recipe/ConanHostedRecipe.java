
package org.globalart.astrum.repository.conan.internal.recipe;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.conan.ConanFormat;
import org.globalart.astrum.repository.conan.internal.ConanEndpoint;
import org.globalart.astrum.repository.conan.internal.common.ConanUserHandlers;
import org.globalart.astrum.repository.conan.internal.hosted.ConanHostedHandlers;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Route;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(ConanHostedRecipe.NAME)
public class ConanHostedRecipe
    extends ConanRecipeSupport
{
  public static final String NAME = "conan-hosted";

  private final ConanHostedHandlers hostedHandlers;

  @Autowired
  public ConanHostedRecipe(
      @Qualifier(HostedType.NAME) final Type type,
      @Qualifier(ConanFormat.NAME) final Format format,
      final ConanHostedHandlers hostedHandlers)
  {
    super(type, format);
    this.hostedHandlers = checkNotNull(hostedHandlers);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(securityFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(httpClientFacet.get());
    repository.attach(maintenanceFacet.get());
    repository.attach(contentFacet.get());
    repository.attach(searchFacet.get());
    repository.attach(browseFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();
    addUserRoutes(builder, repository -> ConanUserHandlers.REVISIONS);
    for (ConanEndpoint endpoint : ConanEndpoint.values()) {
      if (!isUserEndpoint(endpoint)) {
        builder.route(route(endpoint));
      }
    }
    addBrowseUnsupportedRoute(builder);
    builder.defaultHandlers(HttpHandlers.notFound());
    facet.configure(builder.create());
    return facet;
  }

  private Route route(final ConanEndpoint endpoint) {
    Route.Builder route = new Route.Builder().matcher(endpoint.matcher())
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor);
    if (isFileDownload(endpoint)) {
      route.handler(conditionalRequestHandler)
          .handler(partialFetchHandler)
          .handler(contentHeadersHandler)
          .handler(lastDownloadedHandler);
    }
    else if (endpoint.assetKind() != null && !endpoint.isRead()) {
      route.handler(conditionalRequestHandler);
    }
    return route.handler(hostedHandlers.handler).create();
  }
}
