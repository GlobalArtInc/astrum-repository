package org.globalart.astrum.repository.gitlfs.internal.security;

import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.gitlfs.internal.GitLfsObjects;
import org.globalart.astrum.repository.gitlfs.internal.GitLfsPaths;
import org.globalart.astrum.repository.gitlfs.internal.GitLfsPaths.Match;
import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.SecurityFacetSupport;
import org.globalart.astrum.repository.security.SelectorEvaluationCache;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.security.BreadActions;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Maps the Git LFS API onto repository privileges.
 * <p>
 * The batch API is a POST whatever the transfer direction, so the request itself only needs read access and the
 * batch handler checks each object with {@link #isPermitted(Request, String, String)}. Uploading an object needs add,
 * as does creating, verifying and releasing locks. Permissions on objects are checked against their asset path, so
 * content selectors see the same path for the transfer API and for browse.
 */
@Facet.Exposed
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class GitLfsSecurityFacet
    extends SecurityFacetSupport
{
  private final VariableResolverAdapter variableResolverAdapter;

  private final ContentPermissionChecker contentPermissionChecker;

  @Autowired
  public GitLfsSecurityFacet(
      final GitLfsFormatSecurityContributor securityContributor,
      @Qualifier("simple") final VariableResolverAdapter variableResolverAdapter,
      final ContentPermissionChecker contentPermissionChecker)
  {
    super(securityContributor, variableResolverAdapter, contentPermissionChecker);
    this.variableResolverAdapter = checkNotNull(variableResolverAdapter);
    this.contentPermissionChecker = checkNotNull(contentPermissionChecker);
  }

  @Override
  public void ensurePermitted(final Request request, @Nullable final SelectorEvaluationCache selectorCache) {
    super.ensurePermitted(withObjectPath(request), selectorCache);
  }

  /**
   * @return whether the user may perform the action on the object with the given id
   */
  public boolean isPermitted(final Request request, final String oid, final String action) {
    return isPermitted(new Request.Builder().copy(request).path(GitLfsObjects.assetPath(oid)).build(), action);
  }

  /**
   * @return whether the user may perform the action on the path of the request
   */
  public boolean isPermitted(final Request request, final String action) {
    Repository repository = getRepository();
    return contentPermissionChecker.isPermitted(repository.getName(), repository.getFormat().getValue(), action,
        variableResolverAdapter.fromRequest(request, repository));
  }

  @Override
  protected String action(final Request request) {
    Optional<Match> match = GitLfsPaths.match(request.getPath());
    if (match.isEmpty()) {
      return super.action(request);
    }
    String method = request.getAction();
    boolean read = HttpMethods.GET.equals(method) || HttpMethods.HEAD.equals(method);
    return switch (match.get().endpoint()) {
      case BATCH, VERIFY -> BreadActions.READ;
      case OBJECT, ASSET, LOCKS -> read ? BreadActions.READ : BreadActions.ADD;
      case LOCKS_VERIFY, UNLOCK -> BreadActions.ADD;
    };
  }

  private static Request withObjectPath(final Request request) {
    return GitLfsPaths.match(request.getPath())
        .filter(match -> match.endpoint() == GitLfsPaths.Endpoint.OBJECT)
        .map(match -> new Request.Builder().copy(request).path(GitLfsObjects.assetPath(match.value())).build())
        .orElse(request);
  }
}
