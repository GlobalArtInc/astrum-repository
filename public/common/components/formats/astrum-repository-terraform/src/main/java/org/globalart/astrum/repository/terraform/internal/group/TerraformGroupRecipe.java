package org.globalart.astrum.repository.terraform.internal.group;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.group.GroupHandler;
import org.globalart.astrum.repository.terraform.TerraformFormat;
import org.globalart.astrum.repository.terraform.TerraformPaths;
import org.globalart.astrum.repository.terraform.internal.TerraformRecipeSupport;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;

import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;

/**
 * A group merges the version lists of its members; download endpoints and packages come from the first member
 * that has them. Members answer with locations relative to the request, so the group serves the packages too.
 */
@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(TerraformGroupRecipe.NAME)
public class TerraformGroupRecipe
    extends TerraformRecipeSupport
{
  public static final String NAME = "terraform-group";

  private final Provider<GroupFacet> groupFacet;

  private final GroupHandler groupHandler;

  private final TerraformGroupMergeHandler mergeHandler;

  private final TerraformGroupResignHandler resignHandler;

  @Autowired
  public TerraformGroupRecipe(
      @Qualifier(GroupType.NAME) final Type type,
      @Qualifier(TerraformFormat.NAME) final Format format,
      @Qualifier("default") final Provider<GroupFacet> groupFacet,
      @Qualifier("default") final GroupHandler groupHandler,
      final TerraformGroupMergeHandler mergeHandler,
      final TerraformGroupResignHandler resignHandler)
  {
    super(type, format);
    this.groupFacet = checkNotNull(groupFacet);
    this.groupHandler = checkNotNull(groupHandler);
    this.mergeHandler = checkNotNull(mergeHandler);
    this.resignHandler = checkNotNull(resignHandler);
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
    addCommonRoutes(builder);

    builder.route(apiRoute(TerraformPaths.moduleVersions(), mergeHandler));
    builder.route(apiRoute(TerraformPaths.moduleDownload(), resignHandler, groupHandler));
    builder.route(contentRoute(TerraformPaths.moduleArchive(GET, HEAD), groupHandler));

    builder.route(apiRoute(TerraformPaths.providerVersions(), mergeHandler));
    builder.route(apiRoute(TerraformPaths.providerDownload(), resignHandler, groupHandler));
    builder.route(contentRoute(TerraformPaths.providerFile(GET, HEAD), groupHandler));

    builder.route(apiRoute(TerraformPaths.mirrorIndex(), mergeHandler));
    builder.route(apiRoute(TerraformPaths.mirrorVersion(), mergeHandler));

    finish(builder, facet);
    return facet;
  }
}
