
package org.globalart.astrum.repository.apt.datastore.internal.hosted;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.inject.Provider;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.RecipeSupport;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.apt.AptFormat;
import org.globalart.astrum.repository.apt.datastore.AptContentFacet;
import org.globalart.astrum.repository.apt.datastore.internal.data.AptKeyValueFacet;
import org.globalart.astrum.repository.apt.datastore.internal.hosted.metadata.AptHostedMetadataFacet;
import org.globalart.astrum.repository.apt.datastore.internal.metadata.AptMetadataRebuildSchedulerFacet;
import org.globalart.astrum.repository.apt.internal.AptSecurityFacet;
import org.globalart.astrum.repository.apt.internal.gpg.AptSigningFacet;
import org.globalart.astrum.repository.apt.internal.snapshot.AptSnapshotHandler;
import org.globalart.astrum.repository.content.browse.BrowseFacet;
import org.globalart.astrum.repository.content.search.SearchFacet;
import org.globalart.astrum.repository.http.PartialFetchHandler;
import org.globalart.astrum.repository.security.SecurityHandler;
import org.globalart.astrum.repository.types.HostedType;
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

import org.springframework.beans.factory.annotation.Qualifier;

import static org.globalart.astrum.repository.http.HttpHandlers.notFound;
import org.springframework.stereotype.Component;

/**
 * Apt hosted repository recipe.
 *
 * @since 3.31
 */
@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(AptHostedRecipe.NAME)
public class AptHostedRecipe
    extends RecipeSupport
{
  public static final String NAME = "apt-hosted";

  @Autowired
  Provider<AptSecurityFacet> securityFacet;

  @Autowired
  Provider<AptSigningFacet> aptSigningFacet;

  @Autowired
  Provider<ConfigurableViewFacet> viewFacet;

  @Autowired
  Provider<AptContentFacet> aptContentFacet;

  @Autowired
  Provider<AptHostedFacet> aptHostedFacet;

  @Autowired
  Provider<BrowseFacet> browseFacet;

  @Autowired
  Provider<AptLastAssetMaintenanceFacet> maintenanceFacet;

  @Autowired
  Provider<AptHostedSnapshotFacet> hostedSnapshotFacet;

  @Autowired
  Provider<AptKeyValueFacet> aptKeyValueFacet;

  @Autowired
  Provider<AptHostedMetadataFacet> aptHostedMetadataFacet;

  @Autowired
  Provider<AptMetadataRebuildSchedulerFacet> aptMetadataRebuildSchedulerFacet;

  @Autowired
  TimingHandler timingHandler;

  @Autowired
  SecurityHandler securityHandler;

  @Autowired
  ExceptionHandler exceptionHandler;

  @Autowired
  HandlerContributor handlerContributor;

  @Autowired
  ConditionalRequestHandler conditionalRequestHandler;

  @Autowired
  PartialFetchHandler partialFetchHandler;

  @Autowired
  ContentHeadersHandler contentHeadersHandler;

  @Autowired
  LastDownloadedHandler lastDownloadedHandler;

  @Autowired
  AptSnapshotHandler snapshotHandler;

  @Autowired
  AptHostedHandler hostedHandler;

  @Autowired
  Provider<SearchFacet> searchFacet;

  @Autowired
  public AptHostedRecipe(
      @Qualifier(HostedType.NAME) final Type type,
      @Qualifier(AptFormat.NAME) final Format format)
  {
    super(type, format);
  }

  @Override
  public void apply(final Repository repository) throws Exception {
    repository.attach(securityFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(aptSigningFacet.get());
    repository.attach(aptContentFacet.get());
    repository.attach(aptHostedFacet.get());
    repository.attach(maintenanceFacet.get());
    repository.attach(browseFacet.get());
    repository.attach(searchFacet.get());
    repository.attach(hostedSnapshotFacet.get());
    repository.attach(aptKeyValueFacet.get());
    repository.attach(aptHostedMetadataFacet.get());
    repository.attach(aptMetadataRebuildSchedulerFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();

    builder.route(new Route.Builder().matcher(new AlwaysMatcher())
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor)
        .handler(conditionalRequestHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(lastDownloadedHandler)
        .handler(snapshotHandler)
        .handler(hostedHandler)
        .create());

    builder.defaultHandlers(notFound());
    facet.configure(builder.create());
    return facet;
  }
}
