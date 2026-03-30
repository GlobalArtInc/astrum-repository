package org.globalart.astrum.content.npm.internal.group;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.content.npm.internal.NpmRecipeSupport;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.group.GroupHandler;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.npm.internal.NpmFormat;
import org.globalart.astrum.repository.npm.internal.NpmHandlers;
import org.globalart.astrum.repository.npm.internal.search.v1.NpmSearchGroupHandler;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;

import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;
import static org.globalart.astrum.repository.npm.internal.NpmPaths.distTagsMatcher;
import static org.globalart.astrum.repository.npm.internal.NpmPaths.packageMatcher;
import static org.globalart.astrum.repository.npm.internal.NpmPaths.searchV1Matcher;
import static org.globalart.astrum.repository.npm.internal.NpmPaths.tarballMatcher;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(NpmGroupRecipe.NAME)
public class NpmGroupRecipe
    extends NpmRecipeSupport
{
  public static final String NAME = "npm-group";

  private final Provider<GroupFacet> groupFacet;

  private final GroupHandler tarballHandler;

  private final NpmGroupPackageHandler packageHandler;

  private final NpmGroupDistTagsHandler distTagsHandler;

  private final NpmSearchGroupHandler searchHandler;

  @Autowired
  public NpmGroupRecipe(
      @Qualifier(GroupType.NAME) final Type type,
      @Qualifier(NpmFormat.NAME) final Format format,
      @Qualifier("default") final Provider<GroupFacet> groupFacet,
      @Qualifier("default") final GroupHandler tarballHandler,
      final NpmGroupPackageHandler packageHandler,
      final NpmGroupDistTagsHandler distTagsHandler,
      final NpmSearchGroupHandler searchHandler)
  {
    super(type, format);
    this.groupFacet = checkNotNull(groupFacet);
    this.tarballHandler = checkNotNull(tarballHandler);
    this.packageHandler = checkNotNull(packageHandler);
    this.distTagsHandler = checkNotNull(distTagsHandler);
    this.searchHandler = checkNotNull(searchHandler);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(groupFacet.get());
    repository.attach(securityFacet.get());
    repository.attach(tokenFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(contentFacet.get());
    repository.attach(browseFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();

    addBrowseUnsupportedRoute(builder);

    builder.route(searchV1Matcher()
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(NpmHandlers.npmErrorHandler)
        .handler(searchHandler)
        .create());

    createCommonRoutes(builder);

    builder.route(packageMatcher(GET, HEAD)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(NpmHandlers.npmErrorHandler)
        .handler(contentHeadersHandler)
        .handler(packageHandler)
        .create());

    builder.route(tarballMatcher(GET, HEAD)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(NpmHandlers.npmErrorHandler)
        .handler(handlerContributor)
        .handler(tarballHandler)
        .create());

    builder.route(distTagsMatcher(GET)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(NpmHandlers.npmErrorHandler)
        .handler(distTagsHandler)
        .create());

    builder.defaultHandlers(HttpHandlers.badRequest());

    facet.configure(builder.create());

    return facet;
  }
}
