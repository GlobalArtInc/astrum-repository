package org.globalart.astrum.content.npm.internal;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.npm.internal.NpmFormat;
import org.globalart.astrum.repository.npm.internal.NpmHandlers;
import org.globalart.astrum.repository.npm.internal.search.v1.NpmSearchFacetHosted;
import org.globalart.astrum.repository.types.HostedType;
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
import static org.globalart.astrum.repository.http.HttpMethods.PUT;
import static org.globalart.astrum.repository.npm.internal.NpmPaths.distTagsMatcher;
import static org.globalart.astrum.repository.npm.internal.NpmPaths.distTagsUpdateMatcher;
import static org.globalart.astrum.repository.npm.internal.NpmPaths.packageMatcher;
import static org.globalart.astrum.repository.npm.internal.NpmPaths.packageMatcherWithRevision;
import static org.globalart.astrum.repository.npm.internal.NpmPaths.searchV1Matcher;
import static org.globalart.astrum.repository.npm.internal.NpmPaths.tarballMatcher;
import static org.globalart.astrum.repository.npm.internal.NpmPaths.tarballMatcherWithRevision;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(NpmHostedRecipe.NAME)
public class NpmHostedRecipe
    extends NpmRecipeSupport
{
  public static final String NAME = "npm-hosted";

  private final Provider<NpmHostedFacetImpl> hostedFacet;

  private final Provider<NpmSearchFacetHosted> searchV1Facet;

  private final Provider<NpmHostedMaintenanceFacet> maintenanceFacet;

  @Autowired
  public NpmHostedRecipe(
      @Qualifier(HostedType.NAME) final Type type,
      @Qualifier(NpmFormat.NAME) final Format format,
      final Provider<NpmHostedFacetImpl> hostedFacet,
      final Provider<NpmSearchFacetHosted> searchV1Facet,
      final Provider<NpmHostedMaintenanceFacet> maintenanceFacet)
  {
    super(type, format);
    this.hostedFacet = checkNotNull(hostedFacet);
    this.searchV1Facet = checkNotNull(searchV1Facet);
    this.maintenanceFacet = checkNotNull(maintenanceFacet);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(securityFacet.get());
    repository.attach(tokenFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(hostedFacet.get());
    repository.attach(searchV1Facet.get());
    repository.attach(contentFacet.get());
    repository.attach(maintenanceFacet.get());
    repository.attach(searchFacet.get());
    repository.attach(browseFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();

    addBrowseUnsupportedRoute(builder);

    builder.route(searchV1Matcher()
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(NpmHandlers.npmErrorHandler)
        .handler(conditionalRequestHandler)
        .handler(contentHeadersHandler)
        .handler(NpmHandlers.searchV1)
        .create());

    createCommonRoutes(builder);

    builder.route(packageMatcher(GET, HEAD)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(NpmHandlers.npmErrorHandler)
        .handler(conditionalRequestHandler)
        .handler(contentHeadersHandler)
        .handler(lastDownloadedHandler)
        .handler(NpmHandlers.getPackage)
        .create());

    builder.route(packageMatcher(PUT)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(NpmHandlers.npmErrorHandler)
        .handler(handlerContributor)
        .handler(conditionalRequestHandler)
        .handler(contentHeadersHandler)
        .handler(NpmHandlers.putPackage)
        .create());

    builder.route(packageMatcherWithRevision(PUT)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(NpmHandlers.npmErrorHandler)
        .handler(handlerContributor)
        .handler(conditionalRequestHandler)
        .handler(contentHeadersHandler)
        .handler(NpmHandlers.putPackage)
        .create());

    builder.route(packageMatcher(DELETE)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(NpmHandlers.npmErrorHandler)
        .handler(conditionalRequestHandler)
        .handler(contentHeadersHandler)
        .handler(NpmHandlers.deletePackage)
        .create());

    builder.route(packageMatcherWithRevision(DELETE)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(NpmHandlers.npmErrorHandler)
        .handler(conditionalRequestHandler)
        .handler(contentHeadersHandler)
        .handler(NpmHandlers.deletePackage)
        .create());

    builder.route(tarballMatcher(GET, HEAD)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(NpmHandlers.npmErrorHandler)
        .handler(handlerContributor)
        .handler(conditionalRequestHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(lastDownloadedHandler)
        .handler(NpmHandlers.getTarball)
        .create());

    builder.route(tarballMatcher(DELETE)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(NpmHandlers.npmErrorHandler)
        .handler(conditionalRequestHandler)
        .handler(contentHeadersHandler)
        .handler(NpmHandlers.deleteTarball)
        .create());

    builder.route(tarballMatcherWithRevision(DELETE)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(NpmHandlers.npmErrorHandler)
        .handler(conditionalRequestHandler)
        .handler(contentHeadersHandler)
        .handler(NpmHandlers.deleteTarball)
        .create());

    builder.route(distTagsMatcher(GET)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(NpmHandlers.npmErrorHandler)
        .handler(NpmHandlers.getDistTags)
        .create());

    builder.route(distTagsUpdateMatcher(PUT)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(NpmHandlers.npmErrorHandler)
        .handler(NpmHandlers.putDistTags)
        .create());

    builder.route(distTagsUpdateMatcher(DELETE)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(NpmHandlers.npmErrorHandler)
        .handler(NpmHandlers.deleteDistTags)
        .create());

    builder.defaultHandlers(HttpHandlers.badRequest());

    facet.configure(builder.create());

    return facet;
  }
}
