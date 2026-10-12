package org.globalart.astrum.repository.swift.internal.security;

import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.SecurityFacetSupport;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.globalart.astrum.repository.swift.internal.SwiftPaths;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.security.BreadActions;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class SwiftSecurityFacet
    extends SecurityFacetSupport
{
  @Autowired
  public SwiftSecurityFacet(
      final SwiftFormatSecurityContributor securityContributor,
      @Qualifier("simple") final VariableResolverAdapter variableResolverAdapter,
      final ContentPermissionChecker contentPermissionChecker)
  {
    super(securityContributor, variableResolverAdapter, contentPermissionChecker);
  }

  @Override
  protected String action(final Request request) {
    String path = request.getPath();
    String method = request.getAction();
    if (HttpMethods.POST.equals(method)
        && (SwiftPaths.LOGIN.equalsIgnoreCase(path) || "/".equals(path) || path.isEmpty())) {
      return BreadActions.READ;
    }
    if (HttpMethods.PUT.equals(method)) {
      return BreadActions.ADD;
    }
    return super.action(request);
  }
}
