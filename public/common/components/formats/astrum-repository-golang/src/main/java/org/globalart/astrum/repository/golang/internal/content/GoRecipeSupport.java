
package org.globalart.astrum.repository.golang.internal.content;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.RecipeSupport;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.content.browse.BrowseFacet;
import org.globalart.astrum.repository.content.maintenance.LastAssetMaintenanceFacet;
import org.globalart.astrum.repository.content.search.SearchFacet;
import org.globalart.astrum.repository.golang.AssetKind;
import org.globalart.astrum.repository.golang.internal.security.GolangSecurityFacet;
import org.globalart.astrum.repository.http.PartialFetchHandler;
import org.globalart.astrum.repository.httpclient.HttpClientFacet;
import org.globalart.astrum.repository.security.SecurityHandler;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Matcher;
import org.globalart.astrum.repository.view.handlers.ConditionalRequestHandler;
import org.globalart.astrum.repository.view.handlers.ContentHeadersHandler;
import org.globalart.astrum.repository.view.handlers.ExceptionHandler;
import org.globalart.astrum.repository.view.handlers.HandlerContributor;
import org.globalart.astrum.repository.view.handlers.LastDownloadedHandler;
import org.globalart.astrum.repository.view.handlers.TimingHandler;
import org.globalart.astrum.repository.view.matchers.ActionMatcher;
import org.globalart.astrum.repository.view.matchers.logic.LogicMatchers;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher;

import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.golang.AssetKind.INFO;
import static org.globalart.astrum.repository.golang.AssetKind.LATEST;
import static org.globalart.astrum.repository.golang.AssetKind.LIST;
import static org.globalart.astrum.repository.golang.AssetKind.MODULE;
import static org.globalart.astrum.repository.golang.AssetKind.PACKAGE;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;
import static org.globalart.astrum.repository.http.HttpMethods.PUT;

public abstract class GoRecipeSupport
    extends RecipeSupport
{
  protected Provider<GolangSecurityFacet> securityFacet;

  protected Provider<ConfigurableViewFacet> viewFacet;

  protected Provider<GoContentFacet> contentFacet;

  protected Provider<HttpClientFacet> httpClientFacet;

  protected Provider<LastAssetMaintenanceFacet> maintenanceFacet;

  protected Provider<SearchFacet> searchFacet;

  protected Provider<BrowseFacet> browseFacet;

  protected TimingHandler timingHandler;

  protected SecurityHandler securityHandler;

  protected ExceptionHandler exceptionHandler;

  protected HandlerContributor handlerContributor;

  protected ConditionalRequestHandler conditionalRequestHandler;

  protected PartialFetchHandler partialFetchHandler;

  protected ContentHeadersHandler contentHeadersHandler;

  protected LastDownloadedHandler lastDownloadedHandler;

  protected GoRecipeSupport(final Type type, final Format format) {
    super(type, format);
  }

  @Autowired
  public final void setDependencies(
      final Provider<GolangSecurityFacet> securityFacet,
      final Provider<ConfigurableViewFacet> viewFacet,
      final Provider<GoContentFacet> contentFacet,
      final Provider<HttpClientFacet> httpClientFacet,
      final Provider<LastAssetMaintenanceFacet> maintenanceFacet,
      final Provider<SearchFacet> searchFacet,
      final Provider<BrowseFacet> browseFacet,
      final TimingHandler timingHandler,
      final SecurityHandler securityHandler,
      final ExceptionHandler exceptionHandler,
      final HandlerContributor handlerContributor,
      final ConditionalRequestHandler conditionalRequestHandler,
      final PartialFetchHandler partialFetchHandler,
      final ContentHeadersHandler contentHeadersHandler,
      final LastDownloadedHandler lastDownloadedHandler)
  {
    this.securityFacet = checkNotNull(securityFacet);
    this.viewFacet = checkNotNull(viewFacet);
    this.contentFacet = checkNotNull(contentFacet);
    this.httpClientFacet = checkNotNull(httpClientFacet);
    this.maintenanceFacet = checkNotNull(maintenanceFacet);
    this.searchFacet = checkNotNull(searchFacet);
    this.browseFacet = checkNotNull(browseFacet);
    this.timingHandler = checkNotNull(timingHandler);
    this.securityHandler = checkNotNull(securityHandler);
    this.exceptionHandler = checkNotNull(exceptionHandler);
    this.handlerContributor = checkNotNull(handlerContributor);
    this.conditionalRequestHandler = checkNotNull(conditionalRequestHandler);
    this.partialFetchHandler = checkNotNull(partialFetchHandler);
    this.contentHeadersHandler = checkNotNull(contentHeadersHandler);
    this.lastDownloadedHandler = checkNotNull(lastDownloadedHandler);
  }

  protected static Matcher packageMatcher() {
    return createMatcher(PACKAGE, "zip");
  }

  protected static Matcher infoMatcher() {
    return createMatcher(INFO, "info");
  }

  protected static Matcher moduleMatcher() {
    return createMatcher(MODULE, "mod");
  }

  protected static Matcher uploadMatcher() {
    return withAssetKind(LogicMatchers.and(new ActionMatcher(PUT), tokenMatcherForExtension("zip")), PACKAGE);
  }

  protected static Matcher listMatcher() {
    return withAssetKind(
        LogicMatchers.and(new ActionMatcher(GET, HEAD), new TokenMatcher("/{module:.+}/@v/list")), LIST);
  }

  protected static Matcher latestMatcher() {
    return withAssetKind(
        LogicMatchers.and(new ActionMatcher(GET, HEAD), new TokenMatcher("/{module:.+}/@latest")), LATEST);
  }

  protected static Matcher createMatcher(final AssetKind assetKind, final String extension) {
    return withAssetKind(LogicMatchers.and(new ActionMatcher(GET, HEAD), tokenMatcherForExtension(extension)),
        assetKind);
  }

  protected static TokenMatcher tokenMatcherForExtension(final String extension) {
    return new TokenMatcher("/{module:.+}/@v/{version:.+}.{extension:" + extension + "}");
  }

  private static Matcher withAssetKind(final Matcher matcher, final AssetKind assetKind) {
    return LogicMatchers.and(matcher, context -> {
      context.getAttributes().set(AssetKind.class, assetKind);
      return true;
    });
  }
}
