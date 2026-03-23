
package org.globalart.astrum.content.maven.internal.recipe;

import javax.annotation.Nonnull;
import org.springframework.beans.factory.annotation.Autowired;
import jakarta.inject.Provider;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.content.maven.internal.index.MavenContentGroupIndexFacet;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.group.GroupHandler;
import org.globalart.astrum.repository.maven.PurgeUnusedSnapshotsFacet;
import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.globalart.astrum.repository.maven.internal.group.MavenGroupFacet;
import org.globalart.astrum.repository.maven.internal.group.MergingGroupHandler;
import org.globalart.astrum.repository.maven.internal.recipes.Maven2GroupRecipe;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.http.HttpHandlers.notFound;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * @since 3.25
 */
@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(Maven2GroupRecipe.NAME)
public class MavenGroupRecipe
    extends MavenRecipeSupport
    implements Maven2GroupRecipe
{
  private final Provider<MavenContentGroupIndexFacet> mavenGroupIndexFacet;

  private final Provider<MavenGroupFacet> mavenGroupFacet;

  private final Provider<PurgeUnusedSnapshotsFacet> mavenPurgeSnapshotsFacet;

  private final GroupHandler groupHandler;

  private final MergingGroupHandler mergingGroupHandler;

  private final MavenContentIndexGroupHandler indexGroupHandler;

  @Autowired
  public MavenGroupRecipe(
      @Qualifier(GroupType.NAME) final Type type,
      @Qualifier(Maven2Format.NAME) final Format format,
      final Provider<MavenContentGroupIndexFacet> mavenGroupIndexFacet,
      final Provider<MavenGroupFacet> mavenGroupFacet,
      final Provider<PurgeUnusedSnapshotsFacet> mavenPurgeSnapshotsFacet,
      final GroupHandler groupHandler,
      final MergingGroupHandler mergingGroupHandler,
      final MavenContentIndexGroupHandler indexGroupHandler)
  {
    super(type, format);
    this.mavenGroupIndexFacet = checkNotNull(mavenGroupIndexFacet);
    this.mavenGroupFacet = checkNotNull(mavenGroupFacet);
    this.mavenPurgeSnapshotsFacet = checkNotNull(mavenPurgeSnapshotsFacet);
    this.groupHandler = checkNotNull(groupHandler);
    this.mergingGroupHandler = checkNotNull(mergingGroupHandler);
    this.indexGroupHandler = checkNotNull(indexGroupHandler);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(securityFacet.get());
    repository.attach(mavenGroupFacet.get());
    repository.attach(mavenContentFacet.get());
    repository.attach(mavenGroupIndexFacet.get());
    repository.attach(browseFacet.get());
    repository.attach(mavenPurgeSnapshotsFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(mavenMaintenanceFacet.get());
    repository.attach(removeSnapshotsFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();

    addBrowseUnsupportedRoute(builder);

    // Note: partialFetchHandler NOT added for Maven metadata
    builder.route(newMetadataRouteBuilder()
        .handler(contentHeadersHandler)
        .handler(mergingGroupHandler)
        .create());

    builder.route(newIndexRouteBuilder()
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(indexGroupHandler)
        .create());

    builder.route(newArchetypeCatalogRouteBuilder()
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(mergingGroupHandler)
        .create());

    builder.route(newMavenPathRouteBuilder()
        .handler(partialFetchHandler)
        .handler(groupHandler)
        .create());

    builder.defaultHandlers(notFound());

    facet.configure(builder.create());

    return facet;
  }
}
