package org.globalart.astrum.repository.yum.internal.recipe;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;
import org.globalart.astrum.repository.yum.AssetKind;
import org.globalart.astrum.repository.yum.YumFormat;
import org.globalart.astrum.repository.yum.internal.hosted.YumHostedFacet;
import org.globalart.astrum.repository.yum.internal.hosted.YumHostedHandlers;
import org.globalart.astrum.repository.yum.internal.task.YumMetadataRebuildFacet;

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
@Qualifier(YumHostedRecipe.NAME)
public class YumHostedRecipe
    extends YumRecipeSupport
{
  public static final String NAME = "yum-hosted";

  private final Provider<YumHostedFacet> hostedFacet;

  private final Provider<YumMetadataRebuildFacet> rebuildFacet;

  private final YumHostedHandlers hostedHandlers;

  @Autowired
  public YumHostedRecipe(
      @Qualifier(HostedType.NAME) final Type type,
      @Qualifier(YumFormat.NAME) final Format format,
      final Provider<YumHostedFacet> hostedFacet,
      final Provider<YumMetadataRebuildFacet> rebuildFacet,
      final YumHostedHandlers hostedHandlers)
  {
    super(type, format);
    this.hostedFacet = checkNotNull(hostedFacet);
    this.rebuildFacet = checkNotNull(rebuildFacet);
    this.hostedHandlers = checkNotNull(hostedHandlers);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(securityFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(contentFacet.get());
    repository.attach(signingFacet.get());
    repository.attach(hostedFacet.get());
    repository.attach(rebuildFacet.get());
    repository.attach(maintenanceFacet.get());
    repository.attach(searchFacet.get());
    repository.attach(browseFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();

    builder.route(route(repomdMatcher(), AssetKind.REPOMD)
        .handler(conditionalRequestHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(hostedHandlers.getRepomd)
        .create());

    builder.route(route(repodataMatcher(), AssetKind.REPODATA)
        .handler(conditionalRequestHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(hostedHandlers.getMetadata)
        .create());

    builder.route(route(otherMatcher(GET, HEAD), AssetKind.OTHER)
        .handler(conditionalRequestHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(lastDownloadedHandler)
        .handler(hostedHandlers.get)
        .create());

    builder.route(route(otherMatcher(PUT), AssetKind.RPM)
        .handler(contentHeadersHandler)
        .handler(hostedHandlers.put)
        .create());

    builder.route(route(otherMatcher(DELETE), AssetKind.OTHER)
        .handler(hostedHandlers.delete)
        .create());

    addBrowseUnsupportedRoute(builder);

    builder.defaultHandlers(HttpHandlers.notFound());

    facet.configure(builder.create());

    return facet;
  }
}
