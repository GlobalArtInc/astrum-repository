
package org.globalart.astrum.content.maven.internal.recipe;

import javax.annotation.Nonnull;
import org.springframework.beans.factory.annotation.Autowired;
import jakarta.inject.Provider;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.content.maven.internal.index.MavenContentHostedIndexFacet;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.maven.PurgeUnusedSnapshotsFacet;
import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.globalart.astrum.repository.maven.internal.recipes.Maven2HostedRecipe;
import org.globalart.astrum.repository.types.HostedType;
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
@Qualifier(Maven2HostedRecipe.NAME)
public class MavenHostedRecipe
    extends MavenRecipeSupport
    implements Maven2HostedRecipe
{
  private final Provider<MavenContentHostedIndexFacet> mavenIndexFacet;

  private final Provider<PurgeUnusedSnapshotsFacet> mavenPurgeSnapshotsFacet;

  @Autowired
  public MavenHostedRecipe(
      @Qualifier(HostedType.NAME) final Type type,
      @Qualifier(Maven2Format.NAME) final Format format,
      final Provider<MavenContentHostedIndexFacet> mavenIndexFacet,
      final Provider<PurgeUnusedSnapshotsFacet> mavenPurgeSnapshotsFacet)
  {
    super(type, format);
    this.mavenIndexFacet = checkNotNull(mavenIndexFacet);
    this.mavenPurgeSnapshotsFacet = checkNotNull(mavenPurgeSnapshotsFacet);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(securityFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(mavenMetadataRebuildFacet.get());
    repository.attach(mavenContentFacet.get());
    repository.attach(searchFacet.get());
    repository.attach(browseFacet.get());
    repository.attach(mavenArchetypeCatalogFacet.get());
    repository.attach(mavenIndexFacet.get());
    repository.attach(mavenMaintenanceFacet.get());
    repository.attach(removeSnapshotsFacet.get());
    repository.attach(mavenPurgeSnapshotsFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();

    addBrowseUnsupportedRoute(builder);

    // Note: partialFetchHandler NOT added for Maven metadata
    builder.route(newMetadataRouteBuilder()
        .handler(versionPolicyHandler)
        .handler(contentHeadersHandler)
        .handler(lastDownloadedHandler)
        .handler(mavenMetadataRebuildHandler)
        .handler(mavenContentHandler)
        .create());

    builder.route(newIndexRouteBuilder()
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(lastDownloadedHandler)
        .handler(mavenContentHandler)
        .create());

    builder.route(newArchetypeCatalogRouteBuilder()
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(lastDownloadedHandler)
        .handler(archetypeCatalogHandler)
        .handler(mavenContentHandler)
        .create());

    builder.route(newMavenPathRouteBuilder()
        .handler(partialFetchHandler)
        .handler(versionPolicyHandler)
        .handler(contentHeadersHandler)
        .handler(lastDownloadedHandler)
        .handler(mavenContentHandler)
        .create());

    builder.defaultHandlers(notFound());

    facet.configure(builder.create());

    return facet;
  }
}
