
package org.globalart.astrum.repository.cocoapods.internal;

import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.SecurityFacetSupport;
import org.globalart.astrum.repository.security.VariableResolverAdapter;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

@Component
@Qualifier(CocoapodsFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class CocoapodsSecurityFacet
    extends SecurityFacetSupport
{
  @Autowired
  public CocoapodsSecurityFacet(final CocoapodsFormatSecurityContributor securityResource,
                                @Qualifier("simple") final VariableResolverAdapter variableResolverAdapter,
                                final ContentPermissionChecker contentPermissionChecker)
  {
    super(securityResource, variableResolverAdapter, contentPermissionChecker);
  }
}
