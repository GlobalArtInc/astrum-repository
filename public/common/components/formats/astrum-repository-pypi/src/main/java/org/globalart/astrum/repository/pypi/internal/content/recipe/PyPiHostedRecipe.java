package org.globalart.astrum.repository.pypi.internal.content.recipe;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.pypi.internal.AssetKind;
import org.globalart.astrum.repository.pypi.internal.PyPiFormat;
import org.globalart.astrum.repository.pypi.internal.content.PyPiHostedFacet;
import org.globalart.astrum.repository.pypi.internal.content.PyPiHostedHandlers;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;

import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(PyPiHostedRecipe.NAME)
public class PyPiHostedRecipe
    extends PyPiRecipeSupport
{
  public static final String NAME = "pypi-hosted";

  private final Provider<PyPiHostedFacet> hostedFacet;

  private final PyPiHostedHandlers hostedHandlers;

  @Autowired
  public PyPiHostedRecipe(
      @Qualifier(HostedType.NAME) final Type type,
      @Qualifier(PyPiFormat.NAME) final Format format,
      final Provider<PyPiHostedFacet> hostedFacet,
      final PyPiHostedHandlers hostedHandlers)
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
    repository.attach(searchFacet.get());
    repository.attach(contentFacet.get());
    repository.attach(browseFacet.get());
    repository.attach(indexFacet.get());
    repository.attach(maintenanceFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();

    builder.route(rootIndexMatcher()
        .handler(timingHandler)
        .handler(assetKindHandler(AssetKind.ROOT_INDEX))
        .handler(securityHandler)
        .handler(exceptionHandler)
        .handler(conditionalRequestHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(lastDownloadedHandler)
        .handler(hostedHandlers.getRootIndex)
        .create());

    builder.route(indexMatcher()
        .handler(timingHandler)
        .handler(assetKindHandler(AssetKind.INDEX))
        .handler(securityHandler)
        .handler(exceptionHandler)
        .handler(conditionalRequestHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(lastDownloadedHandler)
        .handler(hostedHandlers.getIndex)
        .create());

    builder.route(packagesMatcher()
        .handler(timingHandler)
        .handler(assetKindHandler(AssetKind.PACKAGE))
        .handler(securityHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor)
        .handler(conditionalRequestHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(lastDownloadedHandler)
        .handler(hostedHandlers.getPackage)
        .create());

    builder.route(uploadMatcher()
        .handler(timingHandler)
        .handler(assetKindHandler(AssetKind.PACKAGE))
        .handler(securityHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor)
        .handler(conditionalRequestHandler)
        .handler(contentHeadersHandler)
        .handler(hostedHandlers.postContent)
        .create());

    addBrowseUnsupportedRoute(builder);

    builder.defaultHandlers(HttpHandlers.notFound());

    facet.configure(builder.create());

    return facet;
  }
}
