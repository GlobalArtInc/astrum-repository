package org.globalart.astrum.repository.docker.internal.security;

import javax.annotation.Nullable;

import org.globalart.astrum.common.Description;
import org.globalart.astrum.security.UserPrincipalsHelper;
import org.globalart.astrum.security.anonymous.AnonymousConfiguration;
import org.globalart.astrum.security.anonymous.AnonymousManager;
import org.globalart.astrum.security.authc.apikey.ApiKeyService;
import org.globalart.astrum.security.token.BearerTokenRealm;

import jakarta.inject.Provider;
import org.apache.shiro.authc.AuthenticationInfo;
import org.apache.shiro.authc.AuthenticationToken;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
@Qualifier(DockerTokenRealm.NAME)
@Description("Docker Bearer Token Realm")
public class DockerTokenRealm
    extends BearerTokenRealm
{
  public static final String NAME = "DockerToken";

  private final Provider<AnonymousManager> anonymousManager;

  @Autowired
  public DockerTokenRealm(
      final ApiKeyService keyStore,
      final UserPrincipalsHelper principalsHelper,
      final Provider<AnonymousManager> anonymousManager)
  {
    super(keyStore, principalsHelper, NAME);
    this.anonymousManager = checkNotNull(anonymousManager);
    setAuthenticationCachingEnabled(false);
  }

  @Override
  protected boolean isAnonymousSupported() {
    return anonymousManager.get().isEnabled();
  }

  @Override
  @Nullable
  protected AuthenticationInfo doGetAuthenticationInfo(final AuthenticationToken token) {
    AuthenticationInfo info = super.doGetAuthenticationInfo(token);
    if (info == null) {
      return null;
    }
    AnonymousConfiguration configuration = anonymousManager.get().getConfiguration();
    Object principal = info.getPrincipals().getPrimaryPrincipal();
    boolean anonymousPrincipal = configuration != null && configuration.getUserId() != null
        && configuration.getUserId().equals(principal);
    if (anonymousPrincipal && !anonymousManager.get().isEnabled()) {
      return null;
    }
    return info;
  }
}
