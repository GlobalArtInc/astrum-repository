package org.globalart.astrum.oidc.internal;

import java.util.Collection;

import org.globalart.astrum.common.Description;
import org.globalart.astrum.security.authz.PermissionCachingAuthorizingRealm;
import org.globalart.astrum.security.authz.PrincipalPermissionsCacheFactory;
import org.apache.shiro.authc.AuthenticationInfo;
import org.apache.shiro.authc.AuthenticationToken;
import org.apache.shiro.authc.SimpleAuthenticationInfo;
import org.apache.shiro.authc.credential.AllowAllCredentialsMatcher;
import org.apache.shiro.authz.AuthorizationInfo;
import org.apache.shiro.authz.SimpleAuthorizationInfo;
import org.apache.shiro.subject.PrincipalCollection;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
@Qualifier(OidcRealm.NAME)
@Description("OpenID Connect Realm")
public class OidcRealm
    extends PermissionCachingAuthorizingRealm
{
  public static final String NAME = "OAuth2Realm";

  private final OidcUserStore users;

  @Autowired
  public OidcRealm(final PrincipalPermissionsCacheFactory principalPermissionsCacheFactory, final OidcUserStore users) {
    super(NAME, principalPermissionsCacheFactory);
    this.users = checkNotNull(users);
    setCredentialsMatcher(new AllowAllCredentialsMatcher());
    setAuthenticationCachingEnabled(false);
  }

  @Override
  public boolean supports(final AuthenticationToken token) {
    return token instanceof OidcAuthenticationToken;
  }

  @Override
  protected AuthenticationInfo doGetAuthenticationInfo(final AuthenticationToken token) {
    return new SimpleAuthenticationInfo(token.getPrincipal(), token.getCredentials(), getName());
  }

  @Override
  protected AuthorizationInfo doGetAuthorizationInfo(final PrincipalCollection principals) {
    Collection<?> own = principals.fromRealm(getName());
    if (own.isEmpty()) {
      return null;
    }
    return users.get(own.iterator().next().toString())
        .filter(user -> !user.groups().isEmpty())
        .map(user -> {
          SimpleAuthorizationInfo info = new SimpleAuthorizationInfo();
          info.addRoles(user.groups());
          return (AuthorizationInfo) info;
        })
        .orElse(null);
  }
}
