
package org.globalart.astrum.repository.p2.internal.security;

import org.globalart.astrum.repository.p2.internal.P2Format;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.SecurityFacetSupport;
import org.globalart.astrum.repository.security.VariableResolverAdapter;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

@Component
@Qualifier(P2Format.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class P2SecurityFacet
    extends SecurityFacetSupport
{
  @Autowired
  public P2SecurityFacet(final P2FormatSecurityContributor securityResource,
                        @Qualifier("simple") final VariableResolverAdapter variableResolverAdapter,
                        final ContentPermissionChecker contentPermissionChecker)
  {
    super(securityResource, variableResolverAdapter, contentPermissionChecker);
  }
}
