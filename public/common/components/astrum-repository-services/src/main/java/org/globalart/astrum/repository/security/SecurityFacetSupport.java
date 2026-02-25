
package org.globalart.astrum.repository.security;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.security.BreadActions;
import org.globalart.astrum.selector.VariableSource;

import org.apache.shiro.authz.AuthorizationException;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Support for {@link SecurityFacet} implementations.
 *
 * @since 3.0
 */
public abstract class SecurityFacetSupport
    extends FacetSupport
    implements SecurityFacet
{
  private final RepositoryFormatSecurityContributor securityContributor;

  private final VariableResolverAdapter variableResolverAdapter;

  private final ContentPermissionChecker contentPermissionChecker;

  public SecurityFacetSupport(
      final RepositoryFormatSecurityContributor securityContributor,
      final VariableResolverAdapter variableResolverAdapter,
      final ContentPermissionChecker contentPermissionChecker)
  {
    this.securityContributor = checkNotNull(securityContributor);
    this.variableResolverAdapter = checkNotNull(variableResolverAdapter);
    this.contentPermissionChecker = checkNotNull(contentPermissionChecker);
  }

  @Override
  protected void doInit(final Configuration configuration) throws Exception {
    securityContributor.add(getRepository());
  }

  @Override
  protected void doDelete() throws Exception {
    securityContributor.remove(getRepository());
  }

  @Override
  public void ensurePermitted(final Request request) {
    ensurePermitted(request, null);
  }

  @Override
  public void ensurePermitted(final Request request, @Nullable final SelectorEvaluationCache selectorCache) {
    checkNotNull(request);

    // determine permission action from request
    String action = action(request);

    Repository repo = getRepository();

    VariableSource variableSource = variableResolverAdapter.fromRequest(request, getRepository());
    if (!contentPermissionChecker.isPermitted(repo.getName(), repo.getFormat().getValue(), action, variableSource,
        selectorCache)) {
      throw new AuthorizationException();
    }
  }

  /**
   * Returns BREAD action for request action.
   */
  protected String action(final Request request) {
    switch (request.getAction()) {
      case HttpMethods.OPTIONS:
      case HttpMethods.GET:
      case HttpMethods.HEAD:
      case HttpMethods.TRACE:
        return BreadActions.READ;

      case HttpMethods.POST:
      case HttpMethods.MKCOL:
      case HttpMethods.PATCH:
        return BreadActions.ADD;

      case HttpMethods.PUT:
        return BreadActions.EDIT;

      case HttpMethods.DELETE:
        return BreadActions.DELETE;
    }

    throw new RuntimeException("Unsupported action: " + request.getAction());
  }
}
