package org.globalart.astrum.repository.huggingface.internal.security;

import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.huggingface.internal.HuggingFacePaths;
import org.globalart.astrum.repository.huggingface.internal.HuggingFaceRequest.Kind;
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

/**
 * Path information is asked for with a {@code POST} but only reads.
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class HuggingFaceSecurityFacet
    extends SecurityFacetSupport
{
  @Autowired
  public HuggingFaceSecurityFacet(
      final HuggingFaceFormatSecurityContributor securityContributor,
      @Qualifier("simple") final VariableResolverAdapter variableResolverAdapter,
      final ContentPermissionChecker contentPermissionChecker)
  {
    super(securityContributor, variableResolverAdapter, contentPermissionChecker);
  }

  @Override
  protected String action(final Request request) {
    if (HttpMethods.POST.equals(request.getAction()) && HuggingFacePaths.parse(request.getPath())
        .filter(parsed -> parsed.kind() == Kind.PATHS_INFO)
        .isPresent()) {
      return BreadActions.READ;
    }
    return super.action(request);
  }
}
