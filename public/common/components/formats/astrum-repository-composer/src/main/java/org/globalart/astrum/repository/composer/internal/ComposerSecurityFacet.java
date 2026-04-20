
package org.globalart.astrum.repository.composer.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;
import org.springframework.context.annotation.Scope;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;


import org.globalart.astrum.repository.composer.ComposerFormat;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.SecurityFacetSupport;
import org.globalart.astrum.repository.security.VariableResolverAdapter;

/**
 * Composer format security facet.
 */
@Component
@Qualifier(ComposerFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class ComposerSecurityFacet
    extends SecurityFacetSupport
{
  @Autowired
  public ComposerSecurityFacet(final ComposerFormatSecurityContributor securityContributor,
                            @Qualifier("simple") final VariableResolverAdapter variableResolverAdapter,
                            final ContentPermissionChecker contentPermissionChecker)
  {
    super(securityContributor, variableResolverAdapter, contentPermissionChecker);
  }
}
