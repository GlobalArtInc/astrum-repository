package org.globalart.astrum.repository.pypi.internal.content.recipe;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.group.GroupHandler;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.pypi.internal.AssetKind;
import org.globalart.astrum.repository.pypi.internal.PyPiFormat;
import org.globalart.astrum.repository.pypi.internal.content.PyPiGroupFacet;
import org.globalart.astrum.repository.pypi.internal.content.PyPiIndexGroupHandler;
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
@Qualifier(PyPiGroupRecipe.NAME)
public class PyPiGroupRecipe
    extends PyPiRecipeSupport
{
  public static final String NAME = "pypi-group";

  private final Provider<PyPiGroupFacet> groupFacet;

  private final GroupHandler standardGroupHandler;

  private final PyPiIndexGroupHandler indexGroupHandler;

  @Autowired
  public PyPiGroupRecipe(
      @Qualifier(GroupType.NAME) final Type type,
      @Qualifier(PyPiFormat.NAME) final Format format,
      final Provider<PyPiGroupFacet> groupFacet,
      @Qualifier("default") final GroupHandler standardGroupHandler,
      final PyPiIndexGroupHandler indexGroupHandler)
  {
    super(type, format);
    this.groupFacet = checkNotNull(groupFacet);
    this.standardGroupHandler = checkNotNull(standardGroupHandler);
    this.indexGroupHandler = checkNotNull(indexGroupHandler);
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

    builder.route(rootIndexMatcher()
        .handler(timingHandler)
        .handler(assetKindHandler(AssetKind.ROOT_INDEX))
        .handler(securityHandler)
        .handler(exceptionHandler)
        .handler(conditionalRequestHandler)
        .handler(contentHeadersHandler)
        .handler(lastDownloadedHandler)
        .handler(indexGroupHandler)
        .create());

    builder.route(indexMatcher()
        .handler(timingHandler)
        .handler(assetKindHandler(AssetKind.INDEX))
        .handler(securityHandler)
        .handler(exceptionHandler)
        .handler(conditionalRequestHandler)
        .handler(contentHeadersHandler)
        .handler(lastDownloadedHandler)
        .handler(indexGroupHandler)
        .create());

    builder.route(packagesMatcher()
        .handler(timingHandler)
        .handler(assetKindHandler(AssetKind.PACKAGE))
        .handler(securityHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor)
        .handler(conditionalRequestHandler)
        .handler(standardGroupHandler)
        .create());

    addBrowseUnsupportedRoute(builder);

    builder.defaultHandlers(HttpHandlers.notFound());

    facet.configure(builder.create());

    return facet;
  }
}
