package org.globalart.astrum.repository.nuget.internal.recipe;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.nuget.internal.NugetFormat;
import org.globalart.astrum.repository.nuget.internal.group.NugetGroupHandler;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;

import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.assetDownload;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.autocomplete;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.catalogEntry;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.flatFile;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.flatNuspec;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.flatPackage;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.flatVersions;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.query;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.registrationIndex;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.registrationLeaf;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.registrationPage;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.serviceIndex;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.v2Download;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.v2FindPackagesById;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.v2Metadata;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.v2Packages;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.v2PackagesEntry;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.v2Root;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.v2Search;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(NugetGroupRecipe.NAME)
public class NugetGroupRecipe
    extends NugetRecipeSupport
{
  public static final String NAME = "nuget-group";

  private final Provider<GroupFacet> groupFacet;

  private final NugetGroupHandler groupHandler;

  @Autowired
  public NugetGroupRecipe(
      @Qualifier(GroupType.NAME) final Type type,
      @Qualifier(NugetFormat.NAME) final Format format,
      @Qualifier("default") final Provider<GroupFacet> groupFacet,
      @Qualifier(NugetFormat.NAME) final NugetGroupHandler groupHandler)
  {
    super(type, format);
    this.groupFacet = checkNotNull(groupFacet);
    this.groupHandler = checkNotNull(groupHandler);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(securityFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(groupFacet.get());
    repository.attach(contentFacet.get());
    repository.attach(browseFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();
    builder.route(simpleRoute(v2Root(), groupHandler.v2Root));
    addBrowseUnsupportedRoute(builder);
    builder.route(simpleRoute(serviceIndex(), groupHandler.serviceIndex));
    builder.route(simpleRoute(flatVersions(), groupHandler.versions));
    builder.route(simpleRoute(flatPackage(), groupHandler.firstBinary));
    builder.route(simpleRoute(flatNuspec(), groupHandler.firstBinary));
    builder.route(simpleRoute(flatFile(), groupHandler.firstBinary));
    builder.route(simpleRoute(registrationIndex(), groupHandler.registrationIndex));
    builder.route(simpleRoute(registrationPage(), groupHandler.firstRewritten));
    builder.route(simpleRoute(registrationLeaf(), groupHandler.firstRewritten));
    builder.route(simpleRoute(catalogEntry(), groupHandler.firstRewritten));
    builder.route(simpleRoute(query(), groupHandler.search));
    builder.route(simpleRoute(autocomplete(), groupHandler.autocomplete));
    builder.route(simpleRoute(v2Metadata(), groupHandler.v2Metadata));
    builder.route(simpleRoute(v2FindPackagesById(), groupHandler.v2Feed));
    builder.route(simpleRoute(v2PackagesEntry(), groupHandler.firstRewritten));
    builder.route(simpleRoute(v2Packages(), groupHandler.v2Feed));
    builder.route(simpleRoute(v2Search(), groupHandler.v2Feed));
    builder.route(simpleRoute(v2Download(), groupHandler.firstBinary));
    builder.route(simpleRoute(assetDownload(), groupHandler.firstBinary));
    builder.defaultHandlers(HttpHandlers.notFound());
    facet.configure(builder.create());
    return facet;
  }
}
