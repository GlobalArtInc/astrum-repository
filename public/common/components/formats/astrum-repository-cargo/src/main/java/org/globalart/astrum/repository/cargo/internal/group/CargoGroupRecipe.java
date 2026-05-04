package org.globalart.astrum.repository.cargo.internal.group;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.cargo.CargoFormat;
import org.globalart.astrum.repository.cargo.internal.CargoPaths;
import org.globalart.astrum.repository.cargo.internal.CargoRecipeSupport;
import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.group.GroupHandler;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;

import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(CargoGroupRecipe.NAME)
public class CargoGroupRecipe
    extends CargoRecipeSupport
{
  public static final String NAME = "cargo-group";

  private final Provider<GroupFacet> groupFacet;

  private final GroupHandler groupHandler;

  private final CargoGroupIndexHandler indexHandler;

  private final CargoGroupSearchHandler searchHandler;

  private final CargoGroupWriteHandler writeHandler;

  @Autowired
  public CargoGroupRecipe(
      @Qualifier(GroupType.NAME) final Type type,
      @Qualifier(CargoFormat.NAME) final Format format,
      @Qualifier("default") final Provider<GroupFacet> groupFacet,
      @Qualifier("default") final GroupHandler groupHandler,
      final CargoGroupIndexHandler indexHandler,
      final CargoGroupSearchHandler searchHandler,
      final CargoGroupWriteHandler writeHandler)
  {
    super(type, format);
    this.groupFacet = checkNotNull(groupFacet);
    this.groupHandler = checkNotNull(groupHandler);
    this.indexHandler = checkNotNull(indexHandler);
    this.searchHandler = checkNotNull(searchHandler);
    this.writeHandler = checkNotNull(writeHandler);
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
    builder.route(apiRoute(CargoPaths.publish(), writeHandler));
    builder.route(apiRoute(CargoPaths.yank(), writeHandler));
    builder.route(apiRoute(CargoPaths.unyank(), writeHandler));
    builder.route(apiRoute(CargoPaths.owners(), groupHandler));
    builder.route(apiRoute(CargoPaths.search(), searchHandler));
    builder.route(contentRoute(CargoPaths.download(), groupHandler));
    builder.route(contentRoute(CargoPaths.index(), indexHandler));
    addBrowseUnsupportedRoute(builder);
    builder.defaultHandlers(HttpHandlers.notFound());
    facet.configure(builder.create());
    return facet;
  }
}
