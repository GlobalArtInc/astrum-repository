
package org.globalart.astrum.content.maven.internal.recipe;

import javax.annotation.Nonnull;
import org.springframework.beans.factory.annotation.Autowired;
import jakarta.inject.Provider;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.content.maven.internal.index.MavenContentProxyIndexFacet;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.cache.NegativeCacheFacet;
import org.globalart.astrum.repository.cache.NegativeCacheHandler;
import org.globalart.astrum.repository.httpclient.HttpClientFacet;
import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.globalart.astrum.repository.maven.internal.matcher.MavenLegacyMetaFilesMatcher;
import org.globalart.astrum.repository.maven.internal.recipes.Maven2ProxyRecipe;
import org.globalart.astrum.repository.proxy.ProxyHandler;
import org.globalart.astrum.repository.purge.PurgeUnusedFacet;
import org.globalart.astrum.repository.types.ProxyType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Route.Builder;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.http.HttpHandlers.notFound;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * @since 3.26
 */
@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(Maven2ProxyRecipe.NAME)
public class MavenProxyRecipe
    extends MavenRecipeSupport
    implements Maven2ProxyRecipe
{
  private final Provider<HttpClientFacet> httpClientFacet;

  private final Provider<NegativeCacheFacet> negativeCacheFacet;

  private final Provider<MavenProxyFacet> proxyFacet;

  private final Provider<PurgeUnusedFacet> purgeUnusedFacet;

  private final NegativeCacheHandler negativeCacheHandler;

  private final ProxyHandler proxyHandler;

  private final Provider<MavenContentProxyIndexFacet> mavenProxyIndexFacet;

  @Autowired
  public MavenProxyRecipe(
      @Qualifier(ProxyType.NAME) final Type type,
      @Qualifier(Maven2Format.NAME) final Format format,
      final Provider<HttpClientFacet> httpClientFacet,
      final Provider<NegativeCacheFacet> negativeCacheFacet,
      final Provider<MavenProxyFacet> proxyFacet,
      final Provider<PurgeUnusedFacet> purgeUnusedFacet,
      final NegativeCacheHandler negativeCacheHandler,
      final ProxyHandler proxyHandler,
      final Provider<MavenContentProxyIndexFacet> mavenProxyIndexFacet)
  {
    super(type, format);
    this.httpClientFacet = checkNotNull(httpClientFacet);
    this.negativeCacheFacet = checkNotNull(negativeCacheFacet);
    this.proxyFacet = checkNotNull(proxyFacet);
    this.purgeUnusedFacet = checkNotNull(purgeUnusedFacet);
    this.negativeCacheHandler = checkNotNull(negativeCacheHandler);
    this.proxyHandler = checkNotNull(proxyHandler);
    this.mavenProxyIndexFacet = checkNotNull(mavenProxyIndexFacet);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(securityFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(httpClientFacet.get());
    repository.attach(negativeCacheFacet.get());
    repository.attach(proxyFacet.get());
    repository.attach(mavenContentFacet.get());
    repository.attach(purgeUnusedFacet.get());
    repository.attach(searchFacet.get());
    repository.attach(browseFacet.get());
    repository.attach(mavenProxyIndexFacet.get());
    repository.attach(mavenMaintenanceFacet.get());
    repository.attach(removeSnapshotsFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();

    addBrowseUnsupportedRoute(builder);

    // Note: partialFetchHandler() NOT added for Maven metadata;
    builder.route(newMetadataRouteBuilder()
        .handler(negativeCacheHandler)
        .handler(versionPolicyHandler)
        .handler(contentHeadersHandler)
        .handler(lastDownloadedHandler)
        .handler(proxyHandler)
        .create());

    builder.route(newIndexRouteBuilder()
        .handler(negativeCacheHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(lastDownloadedHandler)
        .handler(proxyHandler)
        .create());

    builder.route(newArchetypeCatalogRouteBuilder()
        .handler(negativeCacheHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(lastDownloadedHandler)
        .handler(proxyHandler)
        .create());

    builder.route(newLegacyMetaFilesRouteBuilder()
        .handler(notFound())
        .create());

    builder.route(newMavenPathRouteBuilder()
        .handler(negativeCacheHandler)
        .handler(partialFetchHandler)
        .handler(versionPolicyHandler)
        .handler(contentHeadersHandler)
        .handler(lastDownloadedHandler)
        .handler(proxyHandler)
        .create());

    builder.defaultHandlers(notFound());

    facet.configure(builder.create());

    return facet;
  }

  private Builder newLegacyMetaFilesRouteBuilder() {
    return new Builder().matcher(new MavenLegacyMetaFilesMatcher(mavenPathParser));
  }
}
