
package org.globalart.astrum.repository.golang.internal.content.group;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.golang.GolangFormat;
import org.globalart.astrum.repository.golang.internal.content.GoRecipeSupport;
import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.group.GroupHandler;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Matcher;
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
@Qualifier(GoGroupRecipe.NAME)
public class GoGroupRecipe
    extends GoRecipeSupport
{
  public static final String NAME = "go-group";

  private final Provider<GroupFacet> groupFacet;

  private final GroupHandler groupHandler;

  private final GoGroupListHandler listHandler;

  @Autowired
  public GoGroupRecipe(
      @Qualifier(GroupType.NAME) final Type type,
      @Qualifier(GolangFormat.NAME) final Format format,
      @Qualifier("default") final Provider<GroupFacet> groupFacet,
      @Qualifier("default") final GroupHandler groupHandler,
      final GoGroupListHandler listHandler)
  {
    super(type, format);
    this.groupFacet = checkNotNull(groupFacet);
    this.groupHandler = checkNotNull(groupHandler);
    this.listHandler = checkNotNull(listHandler);
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
    addBrowseUnsupportedRoute(builder);
    builder.route(route(listMatcher(), listHandler));
    for (Matcher matcher : new Matcher[]{infoMatcher(), packageMatcher(), moduleMatcher(), latestMatcher()}) {
      builder.route(route(matcher, groupHandler));
    }
    builder.defaultHandlers(HttpHandlers.notFound());
    facet.configure(builder.create());
    return facet;
  }

  private Route route(final Matcher matcher, final Handler handler) {
    return new Route.Builder().matcher(matcher)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor)
        .handler(handler)
        .create();
  }
}
