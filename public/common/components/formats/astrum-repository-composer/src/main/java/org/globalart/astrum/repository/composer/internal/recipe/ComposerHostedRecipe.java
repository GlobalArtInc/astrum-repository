package org.globalart.astrum.repository.composer.internal.recipe;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.composer.AssetKind;
import org.globalart.astrum.repository.composer.ComposerFormat;
import org.globalart.astrum.repository.composer.ComposerHostedFacet;
import org.globalart.astrum.repository.composer.internal.hosted.ComposerHostedDownloadHandler;
import org.globalart.astrum.repository.composer.internal.hosted.ComposerHostedUploadHandler;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Matcher;
import org.globalart.astrum.repository.view.Route;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;
import org.globalart.astrum.repository.view.handlers.LastDownloadedHandler;

import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(ComposerHostedRecipe.NAME)
public class ComposerHostedRecipe
    extends ComposerRecipeSupport
{
  public static final String NAME = "composer-hosted";

  private final Provider<ComposerHostedFacet> hostedFacet;

  private final LastDownloadedHandler lastDownloadedHandler;

  private final ComposerHostedUploadHandler uploadHandler;

  private final ComposerHostedDownloadHandler downloadHandler;

  @Autowired
  public ComposerHostedRecipe(
      @Qualifier(HostedType.NAME) final Type type,
      @Qualifier(ComposerFormat.NAME) final Format format,
      final Provider<ComposerHostedFacet> hostedFacet,
      final LastDownloadedHandler lastDownloadedHandler,
      final ComposerHostedUploadHandler uploadHandler,
      final ComposerHostedDownloadHandler downloadHandler)
  {
    super(type, format);
    this.hostedFacet = checkNotNull(hostedFacet);
    this.lastDownloadedHandler = checkNotNull(lastDownloadedHandler);
    this.uploadHandler = checkNotNull(uploadHandler);
    this.downloadHandler = checkNotNull(downloadHandler);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(contentFacet.get());
    repository.attach(securityFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(hostedFacet.get());
    repository.attach(maintenanceFacet.get());
    repository.attach(searchFacet.get());
    repository.attach(browseFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();

    addBrowseUnsupportedRoute(builder);

    builder.route(download(packagesMatcher(), AssetKind.PACKAGES, false));
    builder.route(download(listMatcher(), AssetKind.LIST, false));
    builder.route(download(providerMatcher(), AssetKind.PROVIDER, true));
    builder.route(download(packageMatcher(), AssetKind.PACKAGE, true));
    builder.route(download(zipballMatcher(), AssetKind.ZIPBALL, true));
    builder.route(chain(uploadMatcher(), AssetKind.ZIPBALL).handler(uploadHandler).create());

    builder.defaultHandlers(HttpHandlers.notFound());

    facet.configure(builder.create());

    return facet;
  }

  private Route download(final Matcher matcher, final AssetKind assetKind, final boolean markDownloaded) {
    Route.Builder route = chain(matcher, assetKind);
    if (markDownloaded) {
      route.handler(lastDownloadedHandler);
    }
    return route.handler(downloadHandler).create();
  }

  private Route.Builder chain(final Matcher matcher, final AssetKind assetKind) {
    return route(matcher, assetKind)
        .handler(exceptionHandler)
        .handler(handlerContributor)
        .handler(conditionalRequestHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler);
  }
}
