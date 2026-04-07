package org.globalart.astrum.repository.docker.internal.security;

import org.globalart.astrum.security.SecurityHelper;
import org.globalart.astrum.security.authc.apikey.ApiKeyService;
import org.globalart.astrum.security.token.BearerTokenManager;

import org.apache.shiro.subject.PrincipalCollection;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

@Component
public class DockerTokenManager
    extends BearerTokenManager
{
  @Autowired
  public DockerTokenManager(final ApiKeyService apiKeyService, final SecurityHelper securityHelper) {
    super(apiKeyService, securityHelper, DockerTokenRealm.NAME);
  }

  public String token(final PrincipalCollection principals) {
    return createToken(principals);
  }
}
