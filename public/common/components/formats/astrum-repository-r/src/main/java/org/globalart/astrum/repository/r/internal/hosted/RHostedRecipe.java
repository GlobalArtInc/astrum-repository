
package org.globalart.astrum.repository.r.internal.hosted;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.r.AssetKind;
import org.globalart.astrum.repository.r.RFormat;
import org.globalart.astrum.repository.r.internal.RRecipeSupport;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;
import org.globalart.astrum.repository.view.matchers.logic.LogicMatchers;

import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.http.HttpMethods.DELETE;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(RHostedRecipe.NAME)
public class RHostedRecipe
    extends RRecipeSupport
{
  public static final String NAME = "r-hosted";

  private final Provider<RHostedFacet> hostedFacet;

  private final RHostedHandlers hostedHandlers;

  @Autowired
  public RHostedRecipe(
      @Qualifier(HostedType.NAME) final Type type,
      @Qualifier(RFormat.NAME) final Format format,
      final Provider<RHostedFacet> hostedFacet,
      final RHostedHandlers hostedHandlers)
  {
    super(type, format);
    this.hostedFacet = checkNotNull(hostedFacet);
    this.hostedHandlers = checkNotNull(hostedHandlers);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(securityFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(contentFacet.get());
    repository.attach(hostedFacet.get());
    repository.attach(maintenanceFacet.get());
    repository.attach(searchFacet.get());
    repository.attach(browseFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();

    builder.route(route(packagesMatcher(), AssetKind.PACKAGES)
        .handler(conditionalRequestHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(hostedHandlers.getPackages)
        .create());

    builder.route(notSupportedRoute(LogicMatchers.or(otherPackagesMatcher(), rdsMatcher())));

    builder.route(route(archiveMatcher(GET, HEAD), AssetKind.ARCHIVE)
        .handler(conditionalRequestHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(lastDownloadedHandler)
        .handler(hostedHandlers.getArchive)
        .create());

    builder.route(route(uploadMatcher(), AssetKind.ARCHIVE)
        .handler(contentHeadersHandler)
        .handler(hostedHandlers.putArchive)
        .create());

    builder.route(route(archiveMatcher(DELETE), AssetKind.ARCHIVE)
        .handler(hostedHandlers.deleteArchive)
        .create());

    addBrowseUnsupportedRoute(builder);

    builder.defaultHandlers(HttpHandlers.notFound());

    facet.configure(builder.create());

    return facet;
  }
}
