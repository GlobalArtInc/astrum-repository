package org.globalart.astrum.repository.rubygems.internal.group;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.group.GroupHandler;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.rubygems.RubygemsFormat;
import org.globalart.astrum.repository.rubygems.internal.RubygemsPaths;
import org.globalart.astrum.repository.rubygems.internal.RubygemsRecipeSupport;
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
@Qualifier(RubygemsGroupRecipe.NAME)
public class RubygemsGroupRecipe
    extends RubygemsRecipeSupport
{
  public static final String NAME = "rubygems-group";

  private final Provider<GroupFacet> groupFacet;

  private final GroupHandler groupHandler;

  private final RubygemsGroupIndexHandler indexHandler;

  @Autowired
  public RubygemsGroupRecipe(
      @Qualifier(GroupType.NAME) final Type type,
      @Qualifier(RubygemsFormat.NAME) final Format format,
      @Qualifier("default") final Provider<GroupFacet> groupFacet,
      @Qualifier("default") final GroupHandler groupHandler,
      final RubygemsGroupIndexHandler indexHandler)
  {
    super(type, format);
    this.groupFacet = checkNotNull(groupFacet);
    this.groupHandler = checkNotNull(groupHandler);
    this.indexHandler = checkNotNull(indexHandler);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(groupFacet.get());
    repository.attach(securityFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(contentFacet.get());
    repository.attach(maintenanceFacet.get());
    repository.attach(browseFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();
    addCommonRoutes(builder);
    addReadOnlyRoutes(builder);
    builder.route(indexRoute(RubygemsPaths.versions(), indexHandler));
    builder.route(indexRoute(RubygemsPaths.names(), indexHandler));
    builder.route(indexRoute(RubygemsPaths.info(), indexHandler));
    builder.route(contentRoute(RubygemsPaths.legacySpecs(), indexHandler));
    builder.route(contentRoute(RubygemsPaths.quickSpec(), groupHandler));
    builder.route(contentRoute(RubygemsPaths.gem(), groupHandler));
    addBrowseUnsupportedRoute(builder);
    builder.defaultHandlers(HttpHandlers.notFound());
    facet.configure(builder.create());
    return facet;
  }
}
