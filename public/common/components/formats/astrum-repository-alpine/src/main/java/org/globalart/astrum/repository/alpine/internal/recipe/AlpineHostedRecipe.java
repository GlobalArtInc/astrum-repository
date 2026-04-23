package org.globalart.astrum.repository.alpine.internal.recipe;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.alpine.AlpineFormat;
import org.globalart.astrum.repository.alpine.AssetKind;
import org.globalart.astrum.repository.alpine.internal.hosted.AlpineHostedFacet;
import org.globalart.astrum.repository.alpine.internal.hosted.AlpineHostedHandlers;
import org.globalart.astrum.repository.http.HttpHandlers;
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

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(AlpineHostedRecipe.NAME)
public class AlpineHostedRecipe
    extends AlpineRecipeSupport
{
  public static final String NAME = "alpine-hosted";

  private final Provider<AlpineHostedFacet> hostedFacet;

  private final AlpineHostedHandlers hostedHandlers;

  @Autowired
  public AlpineHostedRecipe(
      @Qualifier(HostedType.NAME) final Type type,
      @Qualifier(AlpineFormat.NAME) final Format format,
      final Provider<AlpineHostedFacet> hostedFacet,
      final AlpineHostedHandlers hostedHandlers)
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
    repository.attach(signingFacet.get());
    repository.attach(hostedFacet.get());
    repository.attach(maintenanceFacet.get());
    repository.attach(searchFacet.get());
    repository.attach(browseFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();

    builder.route(route(indexMatcher(), AssetKind.APK_INDEX)
        .handler(conditionalRequestHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(hostedHandlers.getIndex)
        .create());

    builder.route(route(packageMatcher(GET, HEAD), AssetKind.PACKAGE)
        .handler(conditionalRequestHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(lastDownloadedHandler)
        .handler(hostedHandlers.getPackage)
        .create());

    builder.route(route(packageMatcher(PUT), AssetKind.PACKAGE)
        .handler(contentHeadersHandler)
        .handler(hostedHandlers.putPackage)
        .create());

    builder.route(route(packageMatcher(DELETE), AssetKind.PACKAGE)
        .handler(hostedHandlers.deletePackage)
        .create());

    builder.route(publicKeyRoute());

    addBrowseUnsupportedRoute(builder);

    builder.defaultHandlers(HttpHandlers.notFound());

    facet.configure(builder.create());

    return facet;
  }
}
