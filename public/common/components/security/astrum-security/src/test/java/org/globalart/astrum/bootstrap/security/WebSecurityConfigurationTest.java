package org.globalart.astrum.bootstrap.security;

import java.util.List;

import org.globalart.astrum.security.accesstoken.AccessTokenPrincipal;

import org.apache.shiro.authc.AuthenticationInfo;
import org.apache.shiro.authc.AuthenticationToken;
import org.apache.shiro.authz.AuthorizationInfo;
import org.apache.shiro.authz.SimpleAuthorizationInfo;
import org.apache.shiro.authz.permission.AllPermission;
import org.apache.shiro.mgt.DefaultSecurityManager;
import org.apache.shiro.realm.AuthorizingRealm;
import org.apache.shiro.subject.PrincipalCollection;
import org.apache.shiro.subject.SimplePrincipalCollection;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

public class WebSecurityConfigurationTest
{
  private static final AuthorizingRealm ALMIGHTY_REALM = new AuthorizingRealm()
  {
    @Override
    protected AuthenticationInfo doGetAuthenticationInfo(final AuthenticationToken token) {
      return null;
    }

    @Override
    protected AuthorizationInfo doGetAuthorizationInfo(final PrincipalCollection principals) {
      SimpleAuthorizationInfo info = new SimpleAuthorizationInfo();
      info.addObjectPermission(new AllPermission());
      return info;
    }
  };

  @Test
  public void securityManagerAuthorizerAppliesPrincipalScopes() {
    DefaultSecurityManager securityManager = new DefaultSecurityManager();
    securityManager.setAuthorizer(new WebSecurityConfiguration().authorizer());
    securityManager.setRealms(List.of(ALMIGHTY_REALM));

    SimplePrincipalCollection password = new SimplePrincipalCollection("alice", "test");
    SimplePrincipalCollection readToken = new SimplePrincipalCollection("alice", "test");
    readToken.add(new AccessTokenPrincipal("id", "ci", "read"), "test");

    assertThat(securityManager.isPermitted(password, "astrum:repository-view:raw:raw-hosted:add"), is(true));
    assertThat(securityManager.isPermitted(readToken, "astrum:repository-view:raw:raw-hosted:read"), is(true));
    assertThat(securityManager.isPermitted(readToken, "astrum:repository-view:raw:raw-hosted:add"), is(false));
  }
}
