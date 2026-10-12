package org.globalart.astrum.repository.terraform.internal.security;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.SecurityFacetSupport;
import org.globalart.astrum.repository.security.SelectorEvaluationCache;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.globalart.astrum.repository.terraform.TerraformContentFacet;
import org.globalart.astrum.repository.terraform.TerraformFormat;
import org.globalart.astrum.repository.terraform.internal.DownloadTokens;
import org.globalart.astrum.repository.terraform.internal.TerraformRepositoryConfig;
import org.globalart.astrum.repository.view.Request;

import org.apache.shiro.authz.AuthorizationException;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

@Component
@Qualifier(TerraformFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class TerraformSecurityFacet
    extends SecurityFacetSupport
{
  @Autowired
  public TerraformSecurityFacet(
      final TerraformFormatSecurityContributor securityContributor,
      @Qualifier("simple") final VariableResolverAdapter variableResolverAdapter,
      final ContentPermissionChecker contentPermissionChecker)
  {
    super(securityContributor, variableResolverAdapter, contentPermissionChecker);
  }

  /**
   * A download URL the repository handed out grants reading its one path until it expires, as Terraform fetches
   * provider checksums, signatures and packages without its registry credentials.
   */
  @Override
  public void ensurePermitted(final Request request, @Nullable final SelectorEvaluationCache selectorCache) {
    if (hasValidDownloadToken(request)) {
      return;
    }
    if (TerraformRepositoryConfig.requireAuthentication(getRepository()) && TerraformRepositoryConfig.isAnonymous()) {
      throw new AuthorizationException("Authentication required");
    }
    super.ensurePermitted(request, selectorCache);
  }

  private boolean hasValidDownloadToken(final Request request) {
    String action = request.getAction();
    if (!HttpMethods.GET.equals(action) && !HttpMethods.HEAD.equals(action)) {
      return false;
    }
    String signature = request.getParameters().get(DownloadTokens.SIGNATURE);
    if (signature == null) {
      return false;
    }
    return getRepository().optionalFacet(TerraformContentFacet.class)
        .map(content -> DownloadTokens.of(content)
            .isValid(request.getPath(), request.getParameters().get(DownloadTokens.EXPIRES), signature))
        .orElse(false);
  }
}
