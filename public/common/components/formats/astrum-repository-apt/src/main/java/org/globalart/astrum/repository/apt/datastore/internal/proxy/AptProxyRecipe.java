
package org.globalart.astrum.repository.apt.datastore.internal.proxy;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.RecipeSupport;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.apt.AptFormat;
import org.globalart.astrum.repository.apt.datastore.AptContentFacet;
import org.globalart.astrum.repository.apt.datastore.internal.data.AptKeyValueFacet;
import org.globalart.astrum.repository.apt.datastore.internal.metadata.AptMetadataRebuildSchedulerFacet;
import org.globalart.astrum.repository.apt.datastore.internal.proxy.metadata.AptProxyMetadataFacet;
import org.globalart.astrum.repository.apt.internal.AptSecurityFacet;
import org.globalart.astrum.repository.apt.internal.gpg.AptSigningFacet;
import org.globalart.astrum.repository.apt.internal.snapshot.AptSnapshotHandler;
import org.globalart.astrum.repository.cache.NegativeCacheFacet;
import org.globalart.astrum.repository.cache.NegativeCacheHandler;
import org.globalart.astrum.repository.content.browse.BrowseFacet;
import org.globalart.astrum.repository.content.maintenance.LastAssetMaintenanceFacet;
import org.globalart.astrum.repository.content.search.SearchFacet;
import org.globalart.astrum.repository.http.PartialFetchHandler;
import org.globalart.astrum.repository.httpclient.HttpClientFacet;
import org.globalart.astrum.repository.proxy.ProxyHandler;
import org.globalart.astrum.repository.purge.PurgeUnusedFacet;
import org.globalart.astrum.repository.routing.RoutingRuleHandler;
import org.globalart.astrum.repository.security.SecurityHandler;
import org.globalart.astrum.repository.types.ProxyType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Route;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;
import org.globalart.astrum.repository.view.handlers.ConditionalRequestHandler;
import org.globalart.astrum.repository.view.handlers.ContentHeadersHandler;
import org.globalart.astrum.repository.view.handlers.ExceptionHandler;
import org.globalart.astrum.repository.view.handlers.HandlerContributor;
import org.globalart.astrum.repository.view.handlers.LastDownloadedHandler;
import org.globalart.astrum.repository.view.handlers.TimingHandler;
import org.globalart.astrum.repository.view.matchers.AlwaysMatcher;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.http.HttpHandlers.notFound;

/**
 * Apt proxy repository recipe.
 *
 * @since 3.31
 */
@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(AptProxyRecipe.NAME)
public class AptProxyRecipe
    extends RecipeSupport
{
  public static final String NAME = "apt-proxy";

  @Autowired
  Provider<AptSecurityFacet> securityFacet;

  @Autowired
  Provider<ConfigurableViewFacet> viewFacet;

  @Autowired
  Provider<HttpClientFacet> httpClientFacet;

  @Autowired
  Provider<NegativeCacheFacet> negativeCacheFacet;

  @Autowired
  Provider<AptProxyFacet> proxyFacet;

  @Autowired
  Provider<AptProxySnapshotFacet> proxySnapshotFacet;

  @Autowired
  Provider<PurgeUnusedFacet> purgeUnusedFacet;

  @Qualifier(AptFormat.NAME)
  @Autowired
  Provider<LastAssetMaintenanceFacet> lastAssetMaintenanceFacet;

  @Autowired
  Provider<AptContentFacet> aptContentFacet;

  @Autowired
  Provider<SearchFacet> searchFacet;

  @Autowired
  Provider<BrowseFacet> browseFacet;

  @Qualifier(AptFormat.NAME)
  @Autowired
  Provider<AptKeyValueFacet> keyValueFacet;

  @Qualifier(AptFormat.NAME + "-proxy")
  @Autowired
  Provider<AptProxyMetadataFacet> proxyMetadataFacet;

  @Autowired
  Provider<AptSigningFacet> signingFacet;

  @Autowired
  Provider<AptMetadataRebuildSchedulerFacet> aptMetadataRebuildSchedulerFacet;

  @Autowired
  ExceptionHandler exceptionHandler;

  @Autowired
  TimingHandler timingHandler;

  @Autowired
  SecurityHandler securityHandler;

  @Autowired
  NegativeCacheHandler negativeCacheHandler;

  @Autowired
  PartialFetchHandler partialFetchHandler;

  @Autowired
  ProxyHandler proxyHandler;

  @Autowired
  ConditionalRequestHandler conditionalRequestHandler;

  @Autowired
  ContentHeadersHandler contentHeadersHandler;

  @Autowired
  AptSnapshotHandler snapshotHandler;

  @Autowired
  AptDistributionValidationHandler distributionValidationHandler;

  @Autowired
  AptProxyMetadataHandler proxyMetadataHandler;

  @Autowired
  LastDownloadedHandler lastDownloadedHandler;

  @Autowired
  RoutingRuleHandler routingRuleHandler;

  @Autowired
  HandlerContributor handlerContributor;

  @Autowired
  public AptProxyRecipe(@Qualifier(ProxyType.NAME) final Type type, @Qualifier(AptFormat.NAME) final Format format) {
    super(type, format);
  }

  @Override
  public void apply(final Repository repository) throws Exception {
    repository.attach(securityFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(httpClientFacet.get());
    repository.attach(negativeCacheFacet.get());
    repository.attach(proxyFacet.get());
    repository.attach(proxySnapshotFacet.get());
    repository.attach(aptContentFacet.get());
    repository.attach(browseFacet.get());
    repository.attach(lastAssetMaintenanceFacet.get());
    repository.attach(purgeUnusedFacet.get());
    repository.attach(searchFacet.get());
    repository.attach(keyValueFacet.get());
    repository.attach(signingFacet.get());
    repository.attach(proxyMetadataFacet.get());
    repository.attach(aptMetadataRebuildSchedulerFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();

    builder.route(new Route.Builder().matcher(new AlwaysMatcher())
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(routingRuleHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor)
        .handler(distributionValidationHandler)
        .handler(negativeCacheHandler)
        .handler(conditionalRequestHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(snapshotHandler)
        .handler(lastDownloadedHandler)
        .handler(proxyMetadataHandler)
        .handler(proxyHandler)
        .create());

    builder.defaultHandlers(notFound());
    facet.configure(builder.create());
    return facet;
  }
}
