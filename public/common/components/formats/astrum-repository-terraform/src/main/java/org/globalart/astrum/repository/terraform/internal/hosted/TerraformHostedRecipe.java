package org.globalart.astrum.repository.terraform.internal.hosted;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.terraform.TerraformFormat;
import org.globalart.astrum.repository.terraform.TerraformPaths;
import org.globalart.astrum.repository.terraform.internal.TerraformRecipeSupport;
import org.globalart.astrum.repository.terraform.internal.signing.TerraformSigningFacet;
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
@Qualifier(TerraformHostedRecipe.NAME)
public class TerraformHostedRecipe
    extends TerraformRecipeSupport
{
  public static final String NAME = "terraform-hosted";

  private final Provider<TerraformHostedFacet> hostedFacet;

  private final Provider<TerraformSigningFacet> signingFacet;

  private final Provider<TerraformMaintenanceFacet> maintenanceFacet;

  private final TerraformHostedHandlers handlers;

  @Autowired
  public TerraformHostedRecipe(
      @Qualifier(HostedType.NAME) final Type type,
      @Qualifier(TerraformFormat.NAME) final Format format,
      final Provider<TerraformHostedFacet> hostedFacet,
      final Provider<TerraformSigningFacet> signingFacet,
      final Provider<TerraformMaintenanceFacet> maintenanceFacet,
      final TerraformHostedHandlers handlers)
  {
    super(type, format);
    this.hostedFacet = checkNotNull(hostedFacet);
    this.signingFacet = checkNotNull(signingFacet);
    this.maintenanceFacet = checkNotNull(maintenanceFacet);
    this.handlers = checkNotNull(handlers);
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
    addCommonRoutes(builder);

    builder.route(apiRoute(TerraformPaths.moduleVersions(), handlers.moduleVersions));
    builder.route(apiRoute(TerraformPaths.moduleDownload(), handlers.moduleDownload));
    builder.route(contentRoute(TerraformPaths.moduleArchive(GET, HEAD), lastDownloadedHandler, handlers.getContent));
    builder.route(apiRoute(TerraformPaths.moduleArchive(PUT), contentHeadersHandler, handlers.putModule));
    builder.route(apiRoute(TerraformPaths.moduleArchive(DELETE), handlers.delete));

    builder.route(apiRoute(TerraformPaths.providerVersions(), handlers.providerVersions));
    builder.route(apiRoute(TerraformPaths.providerDownload(), handlers.providerDownload));
    builder.route(contentRoute(TerraformPaths.providerFile(GET, HEAD), lastDownloadedHandler, handlers.getContent));
    builder.route(apiRoute(TerraformPaths.providerFile(PUT), contentHeadersHandler, handlers.putProviderFile));
    builder.route(apiRoute(TerraformPaths.providerFile(DELETE), handlers.delete));

    builder.route(apiRoute(TerraformPaths.mirrorIndex(), handlers.mirrorIndex));
    builder.route(apiRoute(TerraformPaths.mirrorVersion(), handlers.mirrorVersion));

    finish(builder, facet);
    return facet;
  }
}
