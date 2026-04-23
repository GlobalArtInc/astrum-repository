package org.globalart.astrum.repository.alpine.internal.recipe;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.alpine.AlpineFormat;
import org.globalart.astrum.repository.alpine.AssetKind;
import org.globalart.astrum.repository.alpine.internal.group.AlpineGroupFacet;
import org.globalart.astrum.repository.alpine.internal.group.AlpineGroupIndexHandler;
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
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(AlpineGroupRecipe.NAME)
public class AlpineGroupRecipe
    extends AlpineRecipeSupport
{
  public static final String NAME = "alpine-group";

  private final Provider<AlpineGroupFacet> groupFacet;

  private final GroupHandler standardGroupHandler;

  private final AlpineGroupIndexHandler indexHandler;

  @Autowired
  public AlpineGroupRecipe(
      @Qualifier(GroupType.NAME) final Type type,
      @Qualifier(AlpineFormat.NAME) final Format format,
      final Provider<AlpineGroupFacet> groupFacet,
      @Qualifier("default") final GroupHandler standardGroupHandler,
      final AlpineGroupIndexHandler indexHandler)
  {
    super(type, format);
    this.groupFacet = checkNotNull(groupFacet);
    this.standardGroupHandler = checkNotNull(standardGroupHandler);
    this.indexHandler = checkNotNull(indexHandler);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(securityFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(contentFacet.get());
    repository.attach(groupFacet.get());
    repository.attach(signingFacet.get());
    repository.attach(browseFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();

    builder.route(group(indexMatcher(), AssetKind.APK_INDEX, indexHandler));
    builder.route(group(packageMatcher(GET, HEAD), AssetKind.PACKAGE, standardGroupHandler));
    builder.route(publicKeyRoute());

    addBrowseUnsupportedRoute(builder);

    builder.route(group(otherMatcher(), AssetKind.OTHER, standardGroupHandler));

    builder.defaultHandlers(HttpHandlers.notFound());

    facet.configure(builder.create());

    return facet;
  }

  private Route group(final Matcher matcher, final AssetKind assetKind, final Handler handler) {
    return route(matcher, assetKind)
        .handler(conditionalRequestHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(handler)
        .create();
  }
}
