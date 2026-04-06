package org.globalart.astrum.repository.nuget.internal.recipe;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.nuget.internal.NugetFormat;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.assetDownload;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.autocomplete;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.catalogEntry;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.delete;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.flatNuspec;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.flatPackage;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.flatVersions;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.push;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.query;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.registrationIndex;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.registrationLeaf;
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
@Qualifier(NugetHostedRecipe.NAME)
public class NugetHostedRecipe
    extends NugetRecipeSupport
{
  public static final String NAME = "nuget-hosted";

  private final NugetHostedHandlers handlers;

  @Autowired
  public NugetHostedRecipe(
      @Qualifier(HostedType.NAME) final Type type,
      @Qualifier(NugetFormat.NAME) final Format format,
      final NugetHostedHandlers handlers)
  {
    super(type, format);
    this.handlers = checkNotNull(handlers);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(securityFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(contentFacet.get());
    repository.attach(maintenanceFacet.get());
    repository.attach(searchFacet.get());
    repository.attach(browseFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();
    builder.route(simpleRoute(v2Root(), handlers.v2Root));
    addBrowseUnsupportedRoute(builder);
    builder.route(simpleRoute(serviceIndex(), handlers.serviceIndex));
    builder.route(simpleRoute(flatVersions(), handlers.versions));
    builder.route(contentRoute(flatPackage(), handlers.packageContent));
    builder.route(simpleRoute(flatNuspec(), handlers.nuspec));
    builder.route(simpleRoute(registrationIndex(), handlers.registrationIndex));
    builder.route(simpleRoute(registrationLeaf(), handlers.registrationLeaf));
    builder.route(simpleRoute(catalogEntry(), handlers.catalogEntry));
    builder.route(simpleRoute(query(), handlers.search));
    builder.route(simpleRoute(autocomplete(), handlers.autocomplete));
    builder.route(simpleRoute(push(), handlers.push));
    builder.route(simpleRoute(delete(), handlers.delete));
    builder.route(simpleRoute(v2Metadata(), handlers.v2Metadata));
    builder.route(simpleRoute(v2FindPackagesById(), handlers.v2FindPackagesById));
    builder.route(simpleRoute(v2PackagesEntry(), handlers.v2PackagesEntry));
    builder.route(simpleRoute(v2Packages(), handlers.v2Packages));
    builder.route(simpleRoute(v2Search(), handlers.v2Search));
    builder.route(contentRoute(v2Download(), handlers.v2Download));
    builder.route(contentRoute(assetDownload(), handlers.packageContent));
    builder.defaultHandlers(HttpHandlers.notFound());
    facet.configure(builder.create());
    return facet;
  }
}
