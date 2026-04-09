package org.globalart.astrum.repository.helm.internal.content.recipe;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.RecipeSupport;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.content.browse.BrowseFacet;
import org.globalart.astrum.repository.content.maintenance.LastAssetMaintenanceFacet;
import org.globalart.astrum.repository.content.search.SearchFacet;
import org.globalart.astrum.repository.helm.internal.AssetKind;
import org.globalart.astrum.repository.helm.internal.content.HelmContentFacet;
import org.globalart.astrum.repository.helm.internal.security.HelmSecurityFacet;
import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.httpclient.HttpClientFacet;
import org.globalart.astrum.repository.security.SecurityHandler;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Matcher;
import org.globalart.astrum.repository.view.handlers.ConditionalRequestHandler;
import org.globalart.astrum.repository.view.handlers.ContentHeadersHandler;
import org.globalart.astrum.repository.view.handlers.ExceptionHandler;
import org.globalart.astrum.repository.view.handlers.HandlerContributor;
import org.globalart.astrum.repository.view.handlers.TimingHandler;
import org.globalart.astrum.repository.view.matchers.ActionMatcher;
import org.globalart.astrum.repository.view.matchers.LiteralMatcher;
import org.globalart.astrum.repository.view.matchers.logic.LogicMatchers;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher;
import org.globalart.astrum.repository.http.PartialFetchHandler;

import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;

public abstract class HelmRecipeSupport
    extends RecipeSupport
{
  protected Provider<HelmSecurityFacet> securityFacet;

  protected Provider<ConfigurableViewFacet> viewFacet;

  protected Provider<HelmContentFacet> contentFacet;

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

  protected HelmRecipeSupport(final Type type, final Format format) {
    super(type, format);
  }

  @Autowired
  public final void setDependencies(
      final Provider<HelmSecurityFacet> securityFacet,
      final Provider<ConfigurableViewFacet> viewFacet,
      final Provider<HelmContentFacet> contentFacet,
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
      final ContentHeadersHandler contentHeadersHandler)
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
  }

  protected static Matcher withAssetKind(final Matcher matcher, final AssetKind kind) {
    return LogicMatchers.and(matcher, context -> {
      context.getAttributes().set(AssetKind.class, kind);
      return true;
    });
  }

  protected static Matcher indexMatcher() {
    return withAssetKind(LogicMatchers.and(new ActionMatcher(GET, HEAD), new LiteralMatcher("/index.yaml")),
        AssetKind.HELM_INDEX);
  }

  protected static Matcher packageMatcher() {
    return withAssetKind(LogicMatchers.and(new ActionMatcher(GET, HEAD), new TokenMatcher("/{filename:.+}")),
        AssetKind.HELM_PACKAGE);
  }

  protected static TokenMatcher tokenMatcherForExtension(final String extension) {
    return new TokenMatcher("/{filename:.+}.{extension:" + extension + "}");
  }

  protected static Matcher methodMatcher(final String extension, final AssetKind kind, final String... methods) {
    return withAssetKind(LogicMatchers.and(new ActionMatcher(methods), tokenMatcherForExtension(extension)), kind);
  }

  protected static Matcher pushMatcher() {
    return LogicMatchers.and(new ActionMatcher(HttpMethods.POST), new LiteralMatcher("/api/charts"));
  }
}
