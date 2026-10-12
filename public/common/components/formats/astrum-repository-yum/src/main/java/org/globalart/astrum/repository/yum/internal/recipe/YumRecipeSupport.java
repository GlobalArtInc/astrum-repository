package org.globalart.astrum.repository.yum.internal.recipe;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.RecipeSupport;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.content.browse.BrowseFacet;
import org.globalart.astrum.repository.content.maintenance.LastAssetMaintenanceFacet;
import org.globalart.astrum.repository.content.search.SearchFacet;
import org.globalart.astrum.repository.http.PartialFetchHandler;
import org.globalart.astrum.repository.security.SecurityHandler;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Matcher;
import org.globalart.astrum.repository.view.Route;
import org.globalart.astrum.repository.view.handlers.ConditionalRequestHandler;
import org.globalart.astrum.repository.view.handlers.ContentHeadersHandler;
import org.globalart.astrum.repository.view.handlers.ExceptionHandler;
import org.globalart.astrum.repository.view.handlers.HandlerContributor;
import org.globalart.astrum.repository.view.handlers.LastDownloadedHandler;
import org.globalart.astrum.repository.view.handlers.TimingHandler;
import org.globalart.astrum.repository.view.matchers.ActionMatcher;
import org.globalart.astrum.repository.view.matchers.RegexMatcher;
import org.globalart.astrum.repository.view.matchers.logic.LogicMatchers;
import org.globalart.astrum.repository.yum.AssetKind;
import org.globalart.astrum.repository.yum.YumContentFacet;
import org.globalart.astrum.repository.yum.internal.YumSecurityFacet;
import org.globalart.astrum.repository.yum.internal.YumSigningFacet;

import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;

public abstract class YumRecipeSupport
    extends RecipeSupport
{
  protected Provider<YumContentFacet> contentFacet;

  protected Provider<YumSecurityFacet> securityFacet;

  protected Provider<YumSigningFacet> signingFacet;

  protected Provider<LastAssetMaintenanceFacet> maintenanceFacet;

  protected Provider<ConfigurableViewFacet> viewFacet;

  protected Provider<SearchFacet> searchFacet;

  protected Provider<BrowseFacet> browseFacet;

  protected ExceptionHandler exceptionHandler;

  protected TimingHandler timingHandler;

  protected SecurityHandler securityHandler;

  protected PartialFetchHandler partialFetchHandler;

  protected ConditionalRequestHandler conditionalRequestHandler;

  protected ContentHeadersHandler contentHeadersHandler;

  protected HandlerContributor handlerContributor;

  protected LastDownloadedHandler lastDownloadedHandler;

  protected YumRecipeSupport(final Type type, final Format format) {
    super(type, format);
  }

  @Autowired
  public final void setDependencies(
      final Provider<YumContentFacet> contentFacet,
      final Provider<YumSecurityFacet> securityFacet,
      final Provider<YumSigningFacet> signingFacet,
      final Provider<LastAssetMaintenanceFacet> maintenanceFacet,
      final Provider<ConfigurableViewFacet> viewFacet,
      final Provider<SearchFacet> searchFacet,
      final Provider<BrowseFacet> browseFacet,
      final ExceptionHandler exceptionHandler,
      final TimingHandler timingHandler,
      final SecurityHandler securityHandler,
      final PartialFetchHandler partialFetchHandler,
      final ConditionalRequestHandler conditionalRequestHandler,
      final ContentHeadersHandler contentHeadersHandler,
      final HandlerContributor handlerContributor,
      final LastDownloadedHandler lastDownloadedHandler)
  {
    this.contentFacet = checkNotNull(contentFacet);
    this.securityFacet = checkNotNull(securityFacet);
    this.signingFacet = checkNotNull(signingFacet);
    this.maintenanceFacet = checkNotNull(maintenanceFacet);
    this.viewFacet = checkNotNull(viewFacet);
    this.searchFacet = checkNotNull(searchFacet);
    this.browseFacet = checkNotNull(browseFacet);
    this.exceptionHandler = checkNotNull(exceptionHandler);
    this.timingHandler = checkNotNull(timingHandler);
    this.securityHandler = checkNotNull(securityHandler);
    this.partialFetchHandler = checkNotNull(partialFetchHandler);
    this.conditionalRequestHandler = checkNotNull(conditionalRequestHandler);
    this.contentHeadersHandler = checkNotNull(contentHeadersHandler);
    this.handlerContributor = checkNotNull(handlerContributor);
    this.lastDownloadedHandler = checkNotNull(lastDownloadedHandler);
  }

  protected Route.Builder route(final Matcher matcher, final AssetKind assetKind) {
    return new Route.Builder().matcher(matcher)
        .handler(timingHandler)
        .handler(assetKindHandler(assetKind))
        .handler(securityHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor);
  }

  protected static Handler assetKindHandler(final AssetKind assetKind) {
    return context -> {
      context.getAttributes().set(AssetKind.class, assetKind);
      return context.proceed();
    };
  }

  protected static Matcher repomdMatcher() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD), new RegexMatcher("^/(?:.+/)?repodata/repomd\\.xml$"));
  }

  protected static Matcher signingMatcher() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD),
        new RegexMatcher("^/(?:.+/)?repodata/repomd\\.xml\\.(?:asc|key)$"));
  }

  protected static Matcher repodataMatcher() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD), new RegexMatcher("^/(?:.+/)?repodata/[^/]+$"));
  }

  protected static Matcher rpmMatcher(final String... actions) {
    return LogicMatchers.and(new ActionMatcher(actions), new RegexMatcher("^/.+\\.rpm$"));
  }

  protected static Matcher otherMatcher(final String... actions) {
    return LogicMatchers.and(new ActionMatcher(actions), new RegexMatcher("^/.*[^/]$"));
  }
}
