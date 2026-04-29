package org.globalart.astrum.repository.conan.internal.recipe;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.conan.ConanFormat;
import org.globalart.astrum.repository.conan.internal.ConanEndpoint;
import org.globalart.astrum.repository.conan.internal.common.ConanUserHandlers;
import org.globalart.astrum.repository.conan.internal.group.ConanGroupMergingHandler;
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
@Qualifier(ConanGroupRecipe.NAME)
public class ConanGroupRecipe
    extends ConanRecipeSupport
{
  public static final String NAME = "conan-group";

  private final Provider<GroupFacet> groupFacet;

  private final GroupHandler groupHandler;

  private final ConanGroupMergingHandler mergingHandler;

  @Autowired
  public ConanGroupRecipe(
      @Qualifier(GroupType.NAME) final Type type,
      @Qualifier(ConanFormat.NAME) final Format format,
      @Qualifier("default") final Provider<GroupFacet> groupFacet,
      @Qualifier("default") final GroupHandler groupHandler,
      final ConanGroupMergingHandler mergingHandler)
  {
    super(type, format);
    this.groupFacet = checkNotNull(groupFacet);
    this.groupHandler = checkNotNull(groupHandler);
    this.mergingHandler = checkNotNull(mergingHandler);
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
    addUserRoutes(builder, repository -> ConanUserHandlers.REVISIONS);
    for (ConanEndpoint endpoint : ConanEndpoint.values()) {
      if (isUserEndpoint(endpoint) || !endpoint.isRead()) {
        continue;
      }
      builder.route(simpleRoute(endpoint, merges(endpoint) ? mergingHandler : groupHandler));
    }
    addBrowseUnsupportedRoute(builder);
    builder.defaultHandlers(HttpHandlers.notFound());
    facet.configure(builder.create());
    return facet;
  }

  private static boolean merges(final ConanEndpoint endpoint) {
    return switch (endpoint) {
      case SEARCH_V1, SEARCH_V2, RECIPE_LATEST, PACKAGE_LATEST, RECIPE_REVISIONS, PACKAGE_REVISIONS, RECIPE_SEARCH,
           PACKAGES_SEARCH, V1_PACKAGES_SEARCH -> true;
      default -> false;
    };
  }
}
