package org.globalart.astrum.repository.gitlfs.internal.hosted;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.RecipeSupport;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.content.browse.BrowseFacet;
import org.globalart.astrum.repository.content.maintenance.LastAssetMaintenanceFacet;
import org.globalart.astrum.repository.content.search.SearchFacet;
import org.globalart.astrum.repository.gitlfs.GitLfsFormat;
import org.globalart.astrum.repository.gitlfs.internal.GitLfsApiErrorHandler;
import org.globalart.astrum.repository.gitlfs.internal.GitLfsContentFacet;
import org.globalart.astrum.repository.gitlfs.internal.GitLfsPaths;
import org.globalart.astrum.repository.gitlfs.internal.lock.GitLfsLockFacetImpl;
import org.globalart.astrum.repository.gitlfs.internal.lock.GitLfsLockHandlers;
import org.globalart.astrum.repository.gitlfs.internal.security.GitLfsSecurityFacet;
import org.globalart.astrum.repository.http.HttpHandlers;
import org.globalart.astrum.repository.http.PartialFetchHandler;
import org.globalart.astrum.repository.security.SecurityHandler;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.view.ConfigurableViewFacet;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Matcher;
import org.globalart.astrum.repository.view.Route;
import org.globalart.astrum.repository.view.Router;
import org.globalart.astrum.repository.view.ViewFacet;
import org.globalart.astrum.repository.view.handlers.ConditionalRequestHandler;
import org.globalart.astrum.repository.view.handlers.ContentHeadersHandler;
import org.globalart.astrum.repository.view.handlers.ExceptionHandler;
import org.globalart.astrum.repository.view.handlers.HandlerContributor;
import org.globalart.astrum.repository.view.handlers.LastDownloadedHandler;
import org.globalart.astrum.repository.view.handlers.TimingHandler;

import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Hosted Git LFS repository: the batch API, the basic transfer and the file locking API.
 */
@AvailabilityVersion(from = "1.0")
@Component
@Qualifier(GitLfsHostedRecipe.NAME)
public class GitLfsHostedRecipe
    extends RecipeSupport
{
  public static final String NAME = "gitlfs-hosted";

  private Provider<GitLfsSecurityFacet> securityFacet;

  private Provider<ConfigurableViewFacet> viewFacet;

  private Provider<GitLfsContentFacet> contentFacet;

  private Provider<GitLfsLockFacetImpl> lockFacet;

  private Provider<LastAssetMaintenanceFacet> maintenanceFacet;

  private Provider<SearchFacet> searchFacet;

  private Provider<BrowseFacet> browseFacet;

  private TimingHandler timingHandler;

  private SecurityHandler securityHandler;

  private ExceptionHandler exceptionHandler;

  private HandlerContributor handlerContributor;

  private ConditionalRequestHandler conditionalRequestHandler;

  private PartialFetchHandler partialFetchHandler;

  private ContentHeadersHandler contentHeadersHandler;

  private LastDownloadedHandler lastDownloadedHandler;

  private GitLfsApiErrorHandler apiErrorHandler;

  private final GitLfsHostedHandlers hostedHandlers;

  private final GitLfsLockHandlers lockHandlers;

  @Autowired
  public GitLfsHostedRecipe(
      @Qualifier(HostedType.NAME) final Type type,
      @Qualifier(GitLfsFormat.NAME) final Format format,
      final GitLfsHostedHandlers hostedHandlers,
      final GitLfsLockHandlers lockHandlers)
  {
    super(type, format);
    this.hostedHandlers = checkNotNull(hostedHandlers);
    this.lockHandlers = checkNotNull(lockHandlers);
  }

  @Autowired
  public final void setDependencies(
      final Provider<GitLfsSecurityFacet> securityFacet,
      final Provider<ConfigurableViewFacet> viewFacet,
      final Provider<GitLfsContentFacet> contentFacet,
      final Provider<GitLfsLockFacetImpl> lockFacet,
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
      final LastDownloadedHandler lastDownloadedHandler,
      final GitLfsApiErrorHandler apiErrorHandler)
  {
    this.securityFacet = checkNotNull(securityFacet);
    this.viewFacet = checkNotNull(viewFacet);
    this.contentFacet = checkNotNull(contentFacet);
    this.lockFacet = checkNotNull(lockFacet);
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
    this.apiErrorHandler = checkNotNull(apiErrorHandler);
  }

  @Override
  public void apply(@Nonnull final Repository repository) throws Exception {
    repository.attach(securityFacet.get());
    repository.attach(configure(viewFacet.get()));
    repository.attach(contentFacet.get());
    repository.attach(lockFacet.get());
    repository.attach(maintenanceFacet.get());
    repository.attach(searchFacet.get());
    repository.attach(browseFacet.get());
  }

  private ViewFacet configure(final ConfigurableViewFacet facet) {
    Router.Builder builder = new Router.Builder();
    addBrowseUnsupportedRoute(builder);
    builder.route(apiRoute(GitLfsPaths.batch(), hostedHandlers.batch));
    builder.route(apiRoute(GitLfsPaths.verify(), hostedHandlers.verify));
    builder.route(apiRoute(GitLfsPaths.upload(), hostedHandlers.upload));
    builder.route(downloadRoute(GitLfsPaths.download(), hostedHandlers.download));
    builder.route(apiRoute(GitLfsPaths.locks(), lockHandlers.locks));
    builder.route(apiRoute(GitLfsPaths.locksVerify(), lockHandlers.verify));
    builder.route(apiRoute(GitLfsPaths.unlock(), lockHandlers.unlock));
    builder.defaultHandlers(HttpHandlers.notFound());
    facet.configure(builder.create());
    return facet;
  }

  private Route apiRoute(final Matcher matcher, final Handler handler) {
    return new Route.Builder().matcher(matcher)
        .handler(timingHandler)
        .handler(apiErrorHandler)
        .handler(securityHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor)
        .handler(handler)
        .create();
  }

  private Route downloadRoute(final Matcher matcher, final Handler handler) {
    return new Route.Builder().matcher(matcher)
        .handler(timingHandler)
        .handler(securityHandler)
        .handler(exceptionHandler)
        .handler(handlerContributor)
        .handler(conditionalRequestHandler)
        .handler(partialFetchHandler)
        .handler(contentHeadersHandler)
        .handler(lastDownloadedHandler)
        .handler(handler)
        .create();
  }
}
