
package org.globalart.astrum.repository.conan.internal.security;

import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.SecurityFacetSupport;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.security.BreadActions;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class ConanSecurityFacet
    extends SecurityFacetSupport
{
  @Autowired
  public ConanSecurityFacet(
      final ConanFormatSecurityContributor securityContributor,
      @Qualifier("simple") final VariableResolverAdapter variableResolverAdapter,
      final ContentPermissionChecker contentPermissionChecker)
  {
    super(securityContributor, variableResolverAdapter, contentPermissionChecker);
  }

  @Override
  protected String action(final Request request) {
    if (HttpMethods.POST.equals(request.getAction())) {
      String path = request.getPath();
      if (path.endsWith("/packages/delete")) {
        return BreadActions.DELETE;
      }
      if (path.endsWith("/upload_urls")) {
        return BreadActions.EDIT;
      }
    }
    return super.action(request);
  }
}
