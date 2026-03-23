
package org.globalart.astrum.repository.maven.internal;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.SecurityFacetSupport;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Maven 2 security facet.
 *
 * @since 3.0
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class MavenSecurityFacet
    extends SecurityFacetSupport
{
  @Autowired
  public MavenSecurityFacet(
      final MavenFormatSecurityContributor securityContributor,
      @Qualifier(Maven2Format.NAME) final VariableResolverAdapter variableResolverAdapter,
      final ContentPermissionChecker contentPermissionChecker)
  {
    super(securityContributor, variableResolverAdapter, contentPermissionChecker);
  }
}
