
package org.globalart.astrum.repository.npm.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;
import org.springframework.context.annotation.Scope;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;


import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.SecurityFacetSupport;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.security.BreadActions;

/**
 * npm format security facet.
 *
 * @since 3.0
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class NpmSecurityFacet
    extends SecurityFacetSupport
{
  @Autowired
  public NpmSecurityFacet(final NpmFormatSecurityContributor securityContributor,
                          @Qualifier("simple") final VariableResolverAdapter variableResolverAdapter,
                          final ContentPermissionChecker contentPermissionChecker)
  {
    super(securityContributor, variableResolverAdapter, contentPermissionChecker);
  }

  /**
   * Returns BREAD action for request action.
   */
  @Override
  protected String action(final Request request) {
    if (request.getPath().startsWith(NpmPaths.NPM_V1_SECURITY_AUDITS)) {
      return BreadActions.READ;
    }
    return super.action(request);
  }
}
