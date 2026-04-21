package org.globalart.astrum.repository.composer.internal.recipe;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.composer.AssetKind;
import org.globalart.astrum.repository.composer.ComposerFormat;
import org.globalart.astrum.repository.composer.internal.group.ComposerGroupPackageJsonHandler;
import org.globalart.astrum.repository.composer.internal.group.ComposerGroupPackagesJsonHandler;
import org.globalart.astrum.repository.composer.internal.group.ComposerGroupProviderJsonHandler;
import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.group.GroupHandler;
import org.globalart.astrum.repository.http.HttpHandlers;
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

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(ComposerGroupRecipe.NAME)
public class ComposerGroupRecipe
    extends ComposerRecipeSupport
{
  public static final String NAME = "composer-group";

  private final Provider<GroupFacet> groupFacet;

  private final GroupHandler standardGroupHandler;

  private final ComposerGroupPackagesJsonHandler packagesJsonHandler;

  private final ComposerGroupProviderJsonHandler providerJsonHandler;

  private final ComposerGroupPackageJsonHandler packageJsonHandler;

  @Autowired
  public ComposerGroupRecipe(
      @Qualifier(GroupType.NAME) final Type type,
      @Qualifier(ComposerFormat.NAME) final Format format,
      @Qualifier("default") final Provider<GroupFacet> groupFacet,
      @Qualifier("default") final GroupHandler standardGroupHandler,
      final ComposerGroupPackagesJsonHandler packagesJsonHandler,
      final ComposerGroupProviderJsonHandler providerJsonHandler,
      final ComposerGroupPackageJsonHandler packageJsonHandler)
  {
    super(type, format);
    this.groupFacet = checkNotNull(groupFacet);
    this.standardGroupHandler = checkNotNull(standardGroupHandler);
    this.packagesJsonHandler = checkNotNull(packagesJsonHandler);
    this.providerJsonHandler = checkNotNull(providerJsonHandler);
    this.packageJsonHandler = checkNotNull(packageJsonHandler);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(contentFacet.get());
    repository.attach(groupFacet.get());
    repository.attach(securityFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(browseFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();

    addBrowseUnsupportedRoute(builder);

    builder.route(group(packagesMatcher(), AssetKind.PACKAGES, packagesJsonHandler));
    builder.route(group(providerMatcher(), AssetKind.PROVIDER, providerJsonHandler));
    builder.route(group(packageMatcher(), AssetKind.PACKAGE, packageJsonHandler));
    builder.route(group(zipballMatcher(), AssetKind.ZIPBALL, standardGroupHandler));

    builder.defaultHandlers(HttpHandlers.notFound());

    facet.configure(builder.create());

    return facet;
  }

  private Route group(final Matcher matcher, final AssetKind assetKind, final Handler handler) {
    return route(matcher, assetKind)
        .handler(exceptionHandler)
        .handler(handlerContributor)
        .handler(handler)
        .create();
  }
}
