package org.globalart.astrum.repository.conda.internal.recipe;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.conda.AssetKind;
import org.globalart.astrum.repository.conda.CondaFormat;
import org.globalart.astrum.repository.conda.internal.hosted.CondaHostedFacet;
import org.globalart.astrum.repository.conda.internal.hosted.CondaHostedHandlers;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Route;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;

import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.http.HttpMethods.POST;
import static org.globalart.astrum.repository.http.HttpMethods.PUT;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(CondaHostedRecipe.NAME)
public class CondaHostedRecipe
    extends CondaRecipeSupport
{
  public static final String NAME = "conda-hosted";

  private final Provider<CondaHostedFacet> hostedFacet;

  private final CondaHostedHandlers hostedHandlers;

  @Autowired
  public CondaHostedRecipe(
      @Qualifier(HostedType.NAME) final Type type,
      @Qualifier(CondaFormat.NAME) final Format format,
      final Provider<CondaHostedFacet> hostedFacet,
      final CondaHostedHandlers hostedHandlers)
  {
    super(type, format);
    this.hostedFacet = checkNotNull(hostedFacet);
    this.hostedHandlers = checkNotNull(hostedHandlers);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(securityFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(hostedFacet.get());
    repository.attach(contentFacet.get());
    repository.attach(searchFacet.get());
    repository.attach(browseFacet.get());
    repository.attach(maintenanceFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();

    builder.route(route(AssetKind.ARCH_REPODATA_JSON)
        .handler(exceptionHandler)
        .handler(conditionalRequestHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(hostedHandlers.getRepodata)
        .create());

    builder.route(download(AssetKind.ARCH_TAR_PACKAGE));
    builder.route(download(AssetKind.ARCH_CONDA_PACKAGE));
    builder.route(upload(AssetKind.ARCH_TAR_PACKAGE));
    builder.route(upload(AssetKind.ARCH_CONDA_PACKAGE));

    addBrowseUnsupportedRoute(builder);

    builder.defaultHandlers(HttpHandlers.notFound());

    facet.configure(builder.create());

    return facet;
  }

  private Route download(final AssetKind assetKind) {
    return route(assetKind)
        .handler(exceptionHandler)
        .handler(handlerContributor)
        .handler(conditionalRequestHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(lastDownloadedHandler)
        .handler(hostedHandlers.getPackage)
        .create();
  }

  private Route upload(final AssetKind assetKind) {
    return route(assetKind, PUT, POST)
        .handler(exceptionHandler)
        .handler(handlerContributor)
        .handler(conditionalRequestHandler)
        .handler(contentHeadersHandler)
        .handler(hostedHandlers.upload)
        .create();
  }
}
