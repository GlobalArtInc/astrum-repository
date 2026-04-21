package org.globalart.astrum.repository.composer.internal.recipe;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.RecipeSupport;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.composer.AssetKind;
import org.globalart.astrum.repository.composer.ComposerContentFacet;
import org.globalart.astrum.repository.composer.internal.ComposerMaintenanceFacet;
import org.globalart.astrum.repository.composer.internal.ComposerSecurityFacet;
import org.globalart.astrum.repository.content.browse.BrowseFacet;
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
import org.globalart.astrum.repository.view.handlers.TimingHandler;
import org.globalart.astrum.repository.view.matchers.ActionMatcher;
import org.globalart.astrum.repository.view.matchers.LiteralMatcher;
import org.globalart.astrum.repository.view.matchers.logic.LogicMatchers;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher;

import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;
import static org.globalart.astrum.repository.http.HttpMethods.PUT;

public abstract class ComposerRecipeSupport
    extends RecipeSupport
{
  public static final String VENDOR_TOKEN = "vendor";

  public static final String PROJECT_TOKEN = "project";

  public static final String VERSION_TOKEN = "version";

  public static final String NAME_TOKEN = "name";

  public static final String PACKAGE_FIELD_NAME = "package";

  public static final String SOURCE_TYPE_FIELD_NAME = "src-type";

  public static final String SOURCE_URL_FIELD_NAME = "src-url";

  public static final String SOURCE_REFERENCE_FIELD_NAME = "src-ref";

  protected Provider<ComposerContentFacet> contentFacet;

  protected Provider<ComposerMaintenanceFacet> maintenanceFacet;

  protected Provider<ComposerSecurityFacet> securityFacet;

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

  protected ComposerRecipeSupport(final Type type, final Format format) {
    super(type, format);
  }

  @Autowired
  public final void setDependencies(
      final Provider<ComposerContentFacet> contentFacet,
      final Provider<ComposerMaintenanceFacet> maintenanceFacet,
      final Provider<ComposerSecurityFacet> securityFacet,
      final Provider<ConfigurableViewFacet> viewFacet,
      final Provider<SearchFacet> searchFacet,
      final Provider<BrowseFacet> browseFacet,
      final ExceptionHandler exceptionHandler,
      final TimingHandler timingHandler,
      final SecurityHandler securityHandler,
      final PartialFetchHandler partialFetchHandler,
      final ConditionalRequestHandler conditionalRequestHandler,
      final ContentHeadersHandler contentHeadersHandler,
      final HandlerContributor handlerContributor)
  {
    this.contentFacet = checkNotNull(contentFacet);
    this.maintenanceFacet = checkNotNull(maintenanceFacet);
    this.securityFacet = checkNotNull(securityFacet);
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
  }

  protected static Handler assetKindHandler(final AssetKind assetKind) {
    return context -> {
      context.getAttributes().set(AssetKind.class, assetKind);
      return context.proceed();
    };
  }

  protected Route.Builder route(final Matcher matcher, final AssetKind assetKind) {
    return new Route.Builder().matcher(matcher)
        .handler(timingHandler)
        .handler(assetKindHandler(assetKind))
        .handler(securityHandler);
  }

  public static Matcher packagesMatcher() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD), new LiteralMatcher("/packages.json"));
  }

  public static Matcher listMatcher() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD), new LiteralMatcher("/packages/list.json"));
  }

  public static Matcher providerMatcher() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD), new TokenMatcher("/p/{vendor:.+}/{project:.+}.json"));
  }

  public static Matcher packageMatcher() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD), new TokenMatcher("/p2/{vendor:.+}/{project:.+}.json"));
  }

  public static Matcher zipballMatcher() {
    return LogicMatchers.and(new ActionMatcher(GET, HEAD),
        new TokenMatcher("/{vendor:.+}/{project:.+}/{version:.+}/{name:.+}.zip"));
  }

  public static Matcher uploadMatcher() {
    return LogicMatchers.and(new ActionMatcher(PUT),
        new TokenMatcher("/packages/upload/{vendor:.+}/{project:.+}/{version:.+}"));
  }
}
