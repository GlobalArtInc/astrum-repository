
package org.globalart.astrum.content.raw.internal.recipe;

import javax.annotation.Nonnull;
import org.springframework.beans.factory.annotation.Autowired;
import jakarta.inject.Provider;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.group.GroupHandler;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.raw.internal.RawFormat;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Route;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Raw group repository recipe.
 *
 * @since 3.24
 */
@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(RawGroupRecipe.NAME)
public class RawGroupRecipe
    extends RawRecipeSupport
{
  public static final String NAME = "raw-group";

  private final Provider<GroupFacet> groupFacet;

  private final GroupHandler groupHandler;

  @Autowired
  public RawGroupRecipe(
      @Qualifier(GroupType.NAME) final Type type,
      @Qualifier(RawFormat.NAME) final Format format,
      @Qualifier("default") final Provider<GroupFacet> groupFacet,
      @Qualifier("default") final GroupHandler groupHandler)
  {
    super(type, format);
    this.groupFacet = checkNotNull(groupFacet);
    this.groupHandler = checkNotNull(groupHandler);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(securityFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(groupFacet.get());
    repository.attach(contentFacet.get());
    repository.attach(browseFacet.get());
  }

  /**
   * Configure {@link ViewFacet}.
   */
  private ViewFacet configure(final ConfigurableViewFacet viewFacet) {
    Router.Builder builder = new Router.Builder();

    builder.route(new Route.Builder()
        .matcher(PATH_MATCHER)
        .handler(timingHandler)
        .handler(contentDispositionHandler)
        .handler(securityHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor)
        .handler(groupHandler)
        .create());

    builder.defaultHandlers(HttpHandlers.badRequest());

    viewFacet.configure(builder.create());

    return viewFacet;
  }
}
