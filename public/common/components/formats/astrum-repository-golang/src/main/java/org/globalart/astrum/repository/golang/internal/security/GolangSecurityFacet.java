
package org.globalart.astrum.repository.golang.internal.security;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;
import org.springframework.context.annotation.Scope;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;

import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.SecurityFacetSupport;
import org.globalart.astrum.repository.security.VariableResolverAdapter;

@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class GolangSecurityFacet
    extends SecurityFacetSupport
{
  @Autowired
  public GolangSecurityFacet(final GolangFormatSecurityContributor securityResource,
                             @Qualifier("simple") final VariableResolverAdapter variableResolverAdapter,
                             final ContentPermissionChecker contentPermissionChecker)
  {
    super(securityResource, variableResolverAdapter, contentPermissionChecker);
  }
}
