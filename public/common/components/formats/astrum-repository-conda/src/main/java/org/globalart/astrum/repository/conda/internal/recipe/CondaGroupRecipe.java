package org.globalart.astrum.repository.conda.internal.recipe;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.conda.AssetKind;
import org.globalart.astrum.repository.conda.CondaFormat;
import org.globalart.astrum.repository.conda.internal.group.CondaGroupFacet;
import org.globalart.astrum.repository.conda.internal.group.CondaRepodataGroupHandler;
import org.globalart.astrum.repository.group.GroupHandler;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Handler;
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
@Qualifier(CondaGroupRecipe.NAME)
public class CondaGroupRecipe
    extends CondaRecipeSupport
{
  public static final String NAME = "conda-group";

  private final Provider<CondaGroupFacet> groupFacet;

  private final GroupHandler standardGroupHandler;

  private final CondaRepodataGroupHandler repodataGroupHandler;

  @Autowired
  public CondaGroupRecipe(
      @Qualifier(GroupType.NAME) final Type type,
      @Qualifier(CondaFormat.NAME) final Format format,
      final Provider<CondaGroupFacet> groupFacet,
      @Qualifier("default") final GroupHandler standardGroupHandler,
      final CondaRepodataGroupHandler repodataGroupHandler)
  {
    super(type, format);
    this.groupFacet = checkNotNull(groupFacet);
    this.standardGroupHandler = checkNotNull(standardGroupHandler);
    this.repodataGroupHandler = checkNotNull(repodataGroupHandler);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(securityFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(contentFacet.get());
    repository.attach(groupFacet.get());
    repository.attach(browseFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();

    builder.route(groupRoute(AssetKind.ARCH_REPODATA_JSON, repodataGroupHandler));
    builder.route(groupRoute(AssetKind.ARCH_TAR_PACKAGE, standardGroupHandler));
    builder.route(groupRoute(AssetKind.ARCH_CONDA_PACKAGE, standardGroupHandler));
    builder.route(groupRoute(AssetKind.ARCH_INDEX_HTML, standardGroupHandler));
    builder.route(groupRoute(AssetKind.CHANNEL_DATA_JSON, standardGroupHandler));
    builder.route(groupRoute(AssetKind.CHANNEL_RSS_XML, standardGroupHandler));
    builder.route(groupRoute(AssetKind.CHANNEL_INDEX_HTML, standardGroupHandler));

    addBrowseUnsupportedRoute(builder);

    builder.defaultHandlers(HttpHandlers.notFound());

    facet.configure(builder.create());

    return facet;
  }

  private Route groupRoute(final AssetKind assetKind, final Handler handler) {
    Route.Builder route = route(assetKind)
        .handler(exceptionHandler);
    if (assetKind.isPackage()) {
      route.handler(handlerContributor);
    }
    route.handler(conditionalRequestHandler);
    if (handler == repodataGroupHandler) {
      route.handler(partialFetchHandler)
          .handler(contentHeadersHandler);
    }
    return route.handler(handler).create();
  }
}
