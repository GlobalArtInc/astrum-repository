
package org.globalart.astrum.repository.r.internal.group;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.group.GroupHandler;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.r.AssetKind;
import org.globalart.astrum.repository.r.RFormat;
import org.globalart.astrum.repository.r.internal.RRecipeSupport;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Matcher;
import org.globalart.astrum.repository.view.Route;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;

import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(RGroupRecipe.NAME)
public class RGroupRecipe
    extends RRecipeSupport
{
  public static final String NAME = "r-group";

  private final Provider<RGroupFacet> groupFacet;

  private final GroupHandler standardGroupHandler;

  private final PackagesGroupHandler packagesGroupHandler;

  @Autowired
  public RGroupRecipe(
      @Qualifier(GroupType.NAME) final Type type,
      @Qualifier(RFormat.NAME) final Format format,
      final Provider<RGroupFacet> groupFacet,
      @Qualifier("default") final GroupHandler standardGroupHandler,
      final PackagesGroupHandler packagesGroupHandler)
  {
    super(type, format);
    this.groupFacet = checkNotNull(groupFacet);
    this.standardGroupHandler = checkNotNull(standardGroupHandler);
    this.packagesGroupHandler = checkNotNull(packagesGroupHandler);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(groupFacet.get());
    repository.attach(securityFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(contentFacet.get());
    repository.attach(browseFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();

    builder.route(group(packagesMatcher(), AssetKind.PACKAGES, packagesGroupHandler));
    builder.route(notSupportedRoute(otherPackagesMatcher()));
    builder.route(group(rdsMatcher(), AssetKind.RDS_METADATA, standardGroupHandler));
    builder.route(group(archiveMatcher(GET, HEAD), AssetKind.ARCHIVE, standardGroupHandler));

    addBrowseUnsupportedRoute(builder);

    builder.defaultHandlers(HttpHandlers.notFound());

    facet.configure(builder.create());

    return facet;
  }

  private Route group(final Matcher matcher, final AssetKind assetKind, final Handler handler) {
    return route(matcher, assetKind)
        .handler(conditionalRequestHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(handler)
        .create();
  }
}
