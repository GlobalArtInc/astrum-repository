package org.globalart.astrum.repository.yum.internal.recipe;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.group.GroupHandler;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Matcher;
import org.globalart.astrum.repository.view.Route;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;
import org.globalart.astrum.repository.yum.AssetKind;
import org.globalart.astrum.repository.yum.YumFormat;
import org.globalart.astrum.repository.yum.internal.group.YumGroupFacet;
import org.globalart.astrum.repository.yum.internal.group.YumGroupMetadataHandler;
import org.globalart.astrum.repository.yum.internal.group.YumGroupRepomdHandler;

import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;

@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(YumGroupRecipe.NAME)
public class YumGroupRecipe
    extends YumRecipeSupport
{
  public static final String NAME = "yum-group";

  private final Provider<YumGroupFacet> groupFacet;

  private final GroupHandler standardGroupHandler;

  private final YumGroupRepomdHandler repomdHandler;

  private final YumGroupMetadataHandler metadataHandler;

  @Autowired
  public YumGroupRecipe(
      @Qualifier(GroupType.NAME) final Type type,
      @Qualifier(YumFormat.NAME) final Format format,
      final Provider<YumGroupFacet> groupFacet,
      @Qualifier("default") final GroupHandler standardGroupHandler,
      final YumGroupRepomdHandler repomdHandler,
      final YumGroupMetadataHandler metadataHandler)
  {
    super(type, format);
    this.groupFacet = checkNotNull(groupFacet);
    this.standardGroupHandler = checkNotNull(standardGroupHandler);
    this.repomdHandler = checkNotNull(repomdHandler);
    this.metadataHandler = checkNotNull(metadataHandler);
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

    builder.route(group(repomdMatcher(), AssetKind.REPOMD, repomdHandler));
    builder.route(group(repodataMatcher(), AssetKind.REPODATA, metadataHandler));
    builder.route(group(rpmMatcher(GET, HEAD), AssetKind.RPM, standardGroupHandler));

    addBrowseUnsupportedRoute(builder);

    builder.route(group(otherMatcher(GET, HEAD), AssetKind.OTHER, standardGroupHandler));

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
