
package org.globalart.astrum.content.maven.internal.recipe;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.inject.Provider;

import org.globalart.astrum.content.maven.MavenArchetypeCatalogFacet;
import org.globalart.astrum.content.maven.MavenContentFacet;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.RecipeSupport;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.content.browse.BrowseFacet;
import org.globalart.astrum.repository.content.search.SearchFacet;
import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.http.PartialFetchHandler;
import org.globalart.astrum.repository.maven.ContentDispositionHandler;
import org.globalart.astrum.repository.maven.MavenMetadataRebuildFacet;
import org.globalart.astrum.repository.maven.MavenPathParser;
import org.globalart.astrum.repository.maven.RemoveSnapshotsFacet;
import org.globalart.astrum.repository.maven.internal.MavenSecurityFacet;
import org.globalart.astrum.repository.maven.internal.VersionPolicyHandler;
import org.globalart.astrum.repository.maven.internal.matcher.MavenArchetypeCatalogMatcher;
import org.globalart.astrum.repository.maven.internal.matcher.MavenIndexMatcher;
import org.globalart.astrum.repository.maven.internal.matcher.MavenPathMatcher;
import org.globalart.astrum.repository.maven.internal.matcher.MavenRepositoryMetadataMatcher;
import org.globalart.astrum.repository.routing.RoutingRuleHandler;
import org.globalart.astrum.repository.security.SecurityHandler;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Route.Builder;
import org.globalart.astrum.repository.view.handlers.ConditionalRequestHandler;
import org.globalart.astrum.repository.view.handlers.ContentHeadersHandler;
import org.globalart.astrum.repository.view.handlers.ExceptionHandler;
import org.globalart.astrum.repository.view.handlers.HandlerContributor;
import org.globalart.astrum.repository.view.handlers.LastDownloadedHandler;
import org.globalart.astrum.repository.view.handlers.TimingHandler;
import org.globalart.astrum.repository.view.matchers.ActionMatcher;
import org.globalart.astrum.repository.view.matchers.logic.LogicMatchers;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * @since 3.25
 */
public abstract class MavenRecipeSupport
    extends RecipeSupport
{
  protected Provider<ConfigurableViewFacet> viewFacet;

  protected ExceptionHandler exceptionHandler;

  protected TimingHandler timingHandler;

  protected RoutingRuleHandler routingHandler;

  protected SecurityHandler securityHandler;

  protected PartialFetchHandler partialFetchHandler;

  protected ConditionalRequestHandler conditionalRequestHandler;

  protected ContentHeadersHandler contentHeadersHandler;

  protected HandlerContributor handlerContributor;

  protected LastDownloadedHandler lastDownloadedHandler;

  protected VersionPolicyHandler versionPolicyHandler;

  protected Provider<MavenSecurityFacet> securityFacet;

  protected MavenPathParser mavenPathParser;

  protected MavenContentHandler mavenContentHandler;

  protected MavenArchetypeCatalogHandler archetypeCatalogHandler;

  protected MavenMetadataRebuildHandler mavenMetadataRebuildHandler;

  protected Provider<MavenContentFacet> mavenContentFacet;

  protected Provider<SearchFacet> searchFacet;

  protected Provider<BrowseFacet> browseFacet;

  protected Provider<MavenArchetypeCatalogFacet> mavenArchetypeCatalogFacet;

  protected Provider<MavenMetadataRebuildFacet> mavenMetadataRebuildFacet;

  protected Provider<MavenMaintenanceFacet> mavenMaintenanceFacet;

  protected Provider<RemoveSnapshotsFacet> removeSnapshotsFacet;

  protected ContentDispositionHandler contentDispositionHandler;

  protected MavenRecipeSupport(final Type type, final Format format) {
    super(type, format);
  }

  public Builder newArchetypeCatalogRouteBuilder() {
    return new Builder().matcher(new MavenArchetypeCatalogMatcher(mavenPathParser))
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(routingHandler)
        .handler(exceptionHandler)
        .handler(conditionalRequestHandler);
  }

  /**
   * Only GET, HEAD actions allowed, as nothing publishes the binary index, only consumes.
   */
  public Builder newIndexRouteBuilder() {
    return new Builder().matcher(LogicMatchers.and(new MavenIndexMatcher(mavenPathParser),
        LogicMatchers.or(new ActionMatcher(HttpMethods.GET), new ActionMatcher(HttpMethods.HEAD))))
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(routingHandler)
        .handler(exceptionHandler)
        .handler(conditionalRequestHandler);
  }

  public Builder newMavenPathRouteBuilder() {
    return new Builder().matcher(new MavenPathMatcher(mavenPathParser))
        .handler(timingHandler)
        .handler(contentDispositionHandler)
        .handler(securityHandler)
        .handler(routingHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor)
        .handler(conditionalRequestHandler);
  }

  public Builder newMetadataRouteBuilder() {
    return new Builder().matcher(new MavenRepositoryMetadataMatcher(mavenPathParser))
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(routingHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor)
        .handler(conditionalRequestHandler);
  }

  @Autowired
  public void setArchetypeCatalogHandler(final MavenArchetypeCatalogHandler archetypeCatalogHandler) {
    this.archetypeCatalogHandler = checkNotNull(archetypeCatalogHandler);
  }

  @Autowired
  public void setBrowseFacet(final Provider<BrowseFacet> browseFacet) {
    this.browseFacet = checkNotNull(browseFacet);
  }

  @Autowired
  public void setConditionalRequestHandler(final ConditionalRequestHandler conditionalRequestHandler) {
    this.conditionalRequestHandler = checkNotNull(conditionalRequestHandler);
  }

  @Autowired
  public void setContentDispositionHandler(final ContentDispositionHandler contentDispositionHandler) {
    this.contentDispositionHandler = checkNotNull(contentDispositionHandler);
  }

  @Autowired
  public void setContentHeadersHandler(final ContentHeadersHandler contentHeadersHandler) {
    this.contentHeadersHandler = checkNotNull(contentHeadersHandler);
  }

  @Autowired
  public void setExceptionHandler(final ExceptionHandler exceptionHandler) {
    this.exceptionHandler = checkNotNull(exceptionHandler);
  }

  @Autowired
  public void setHandlerContributor(final HandlerContributor handlerContributor) {
    this.handlerContributor = checkNotNull(handlerContributor);
  }

  @Autowired
  public void setLastDownloadedHandler(final LastDownloadedHandler lastDownloadedHandler) {
    this.lastDownloadedHandler = checkNotNull(lastDownloadedHandler);
  }

  @Autowired
  public void setMavenArchetypeCatalogFacet(final Provider<MavenArchetypeCatalogFacet> mavenArchetypeCatalogFacet) {
    this.mavenArchetypeCatalogFacet = checkNotNull(mavenArchetypeCatalogFacet);
  }

  @Autowired
  public void setMavenContentFacet(final Provider<MavenContentFacet> mavenContentFacet) {
    this.mavenContentFacet = checkNotNull(mavenContentFacet);
  }

  @Autowired
  public void setMavenContentHandler(final MavenContentHandler mavenContentHandler) {
    this.mavenContentHandler = checkNotNull(mavenContentHandler);
  }

  @Autowired
  public void setMavenMaintenanceFacet(final Provider<MavenMaintenanceFacet> mavenMaintenanceFacet) {
    this.mavenMaintenanceFacet = checkNotNull(mavenMaintenanceFacet);
  }

  @Autowired
  public void setMavenMetadataRebuildFacet(final Provider<MavenMetadataRebuildFacet> mavenMetadataRebuildFacet) {
    this.mavenMetadataRebuildFacet = checkNotNull(mavenMetadataRebuildFacet);
  }

  @Autowired
  public void setMavenMetadataRebuildHandler(final MavenMetadataRebuildHandler mavenMetadataRebuildHandler) {
    this.mavenMetadataRebuildHandler = checkNotNull(mavenMetadataRebuildHandler);
  }

  @Autowired
  public void setMavenPathParser(final MavenPathParser mavenPathParser) {
    this.mavenPathParser = checkNotNull(mavenPathParser);
  }

  @Autowired
  public void setPartialFetchHandler(final PartialFetchHandler partialFetchHandler) {
    this.partialFetchHandler = checkNotNull(partialFetchHandler);
  }

  @Autowired
  public void setRemoveSnapshotsFacet(final Provider<RemoveSnapshotsFacet> removeSnapshotsFacet) {
    this.removeSnapshotsFacet = checkNotNull(removeSnapshotsFacet);
  }

  @Autowired
  public void setRoutingHandler(final RoutingRuleHandler routingHandler) {
    this.routingHandler = checkNotNull(routingHandler);
  }

  @Autowired
  public void setSearchFacet(final Provider<SearchFacet> searchFacet) {
    this.searchFacet = checkNotNull(searchFacet);
  }

  @Autowired
  public void setSecurityFacet(final Provider<MavenSecurityFacet> securityFacet) {
    this.securityFacet = checkNotNull(securityFacet);
  }

  @Autowired
  public void setSecurityHandler(final SecurityHandler securityHandler) {
    this.securityHandler = checkNotNull(securityHandler);
  }

  @Autowired
  public void setTimingHandler(final TimingHandler timingHandler) {
    this.timingHandler = checkNotNull(timingHandler);
  }

  @Autowired
  public void setVersionPolicyHandler(final VersionPolicyHandler versionPolicyHandler) {
    this.versionPolicyHandler = checkNotNull(versionPolicyHandler);
  }

  @Autowired
  public void setViewFacet(final Provider<ConfigurableViewFacet> viewFacet) {
    this.viewFacet = checkNotNull(viewFacet);
  }
}
