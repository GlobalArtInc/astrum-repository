package org.globalart.astrum.repository.docker.internal.security;

import org.globalart.astrum.repository.docker.DockerFormat;
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
@Qualifier(DockerFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class DockerSecurityFacet
    extends SecurityFacetSupport
{
  @Autowired
  public DockerSecurityFacet(
      final DockerFormatSecurityContributor securityContributor,
      @Qualifier("simple") final VariableResolverAdapter variableResolverAdapter,
      final ContentPermissionChecker contentPermissionChecker)
  {
    super(securityContributor, variableResolverAdapter, contentPermissionChecker);
  }

  @Override
  protected String action(final Request request) {
    switch (request.getAction()) {
      case HttpMethods.POST:
      case HttpMethods.PATCH:
      case HttpMethods.PUT:
        return BreadActions.ADD;
      default:
        return super.action(request);
    }
  }
}
