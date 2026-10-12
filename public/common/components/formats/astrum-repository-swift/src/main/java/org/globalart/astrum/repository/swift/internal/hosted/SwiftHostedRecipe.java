package org.globalart.astrum.repository.swift.internal.hosted;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.swift.SwiftFormat;
import org.globalart.astrum.repository.swift.internal.SwiftPaths;
import org.globalart.astrum.repository.swift.internal.SwiftRecipeSupport;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(SwiftHostedRecipe.NAME)
public class SwiftHostedRecipe
    extends SwiftRecipeSupport
{
  public static final String NAME = "swift-hosted";

  private final SwiftHostedHandlers hostedHandlers;

  @Autowired
  public SwiftHostedRecipe(
      @Qualifier(HostedType.NAME) final Type type,
      @Qualifier(SwiftFormat.NAME) final Format format,
      final SwiftHostedHandlers hostedHandlers)
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
    addCommonRoutes(builder);
    builder.route(apiRoute(SwiftPaths.identifiers(), hostedHandlers.identifiers));
    builder.route(apiRoute(SwiftPaths.publish(), hostedHandlers.publish));
    builder.route(contentRoute(SwiftPaths.archive(), lastDownloadedHandler, hostedHandlers.archive));
    builder.route(contentRoute(SwiftPaths.manifest(), lastDownloadedHandler, hostedHandlers.manifest));
    builder.route(contentRoute(SwiftPaths.releases(), hostedHandlers.releases));
    builder.route(contentRoute(SwiftPaths.release(), hostedHandlers.release));
    addDefaultRoutes(builder);
    facet.configure(builder.create());
    return facet;
  }
}
