
package org.globalart.astrum.repository.r.internal;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.RecipeSupport;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.content.browse.BrowseFacet;
import org.globalart.astrum.repository.content.search.SearchFacet;
import org.globalart.astrum.repository.http.PartialFetchHandler;
import org.globalart.astrum.repository.r.AssetKind;
import org.globalart.astrum.repository.r.RContentFacet;
import org.globalart.astrum.repository.r.internal.security.RSecurityFacet;
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
import org.globalart.astrum.repository.view.matchers.SuffixMatcher;
import org.globalart.astrum.repository.view.matchers.logic.LogicMatchers;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher;

import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;
import static org.globalart.astrum.repository.http.HttpMethods.PUT;

public abstract class RRecipeSupport
    extends RecipeSupport
{
  protected Provider<RContentFacet> contentFacet;

  protected Provider<RSecurityFacet> securityFacet;

  protected Provider<RMaintenanceFacet> maintenanceFacet;

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

  protected RCommonHandlers commonHandlers;

  protected RRecipeSupport(final Type type, final Format format) {
    super(type, format);
  }

  @Autowired
  public final void setDependencies(
      final Provider<RContentFacet> contentFacet,
      final Provider<RSecurityFacet> securityFacet,
      final Provider<RMaintenanceFacet> maintenanceFacet,
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
      final LastDownloadedHandler lastDownloadedHandler,
      final RCommonHandlers commonHandlers)
  {
    this.contentFacet = checkNotNull(contentFacet);
    this.securityFacet = checkNotNull(securityFacet);
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
    this.commonHandlers = checkNotNull(commonHandlers);
  }

  protected Route.Builder route(final Matcher matcher, final AssetKind assetKind) {
    return new Route.Builder().matcher(matcher)
        .handler(timingHandler)
        .handler(assetKindHandler(assetKind))
        .handler(securityHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor);
  }

  protected Route notSupportedRoute(final Matcher matcher) {
    return new Route.Builder().matcher(matcher)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(commonHandlers.notSupportedMetadataRequest)
        .create();
  }

  protected static Handler assetKindHandler(final AssetKind assetKind) {
    return context -> {
      context.getAttributes().set(AssetKind.class, assetKind);
      return context.proceed();
    };
  }

  protected static Matcher packagesMatcher() {
    return LogicMatchers.and(
        new ActionMatcher(GET, HEAD),
        LogicMatchers.or(
            new TokenMatcher("/{path:.+}/PACKAGES"),
            new TokenMatcher("/{path:.+}/PACKAGES.gz")));
  }

  protected static Matcher allPackagesMatcher() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD), new TokenMatcher("/{path:.+}/PACKAGES{extension:.*}"));
  }

  protected static Matcher otherPackagesMatcher() {
    return LogicMatchers.and(allPackagesMatcher(), LogicMatchers.not(packagesMatcher()));
  }

  protected static Matcher rdsMatcher() {
    return LogicMatchers.and(
        new ActionMatcher(GET, HEAD),
        new TokenMatcher("/{path:.+}"),
        new SuffixMatcher(".rds"));
  }

  protected static Matcher archiveMatcher(final String... actions) {
    return LogicMatchers.and(
        new ActionMatcher(actions),
        new TokenMatcher("/{path:.+}/{filename:[^/]+}"),
        LogicMatchers.or(
            new SuffixMatcher(".zip"),
            new SuffixMatcher(".tgz"),
            new SuffixMatcher(".tar.gz")));
  }

  protected static Matcher uploadMatcher() {
    return LogicMatchers.and(new ActionMatcher(PUT), new TokenMatcher("/{path:.+}"));
  }
}
