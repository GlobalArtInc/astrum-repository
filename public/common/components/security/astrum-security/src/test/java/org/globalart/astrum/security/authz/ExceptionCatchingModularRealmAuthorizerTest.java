
package org.globalart.astrum.security.authz;

import java.util.Collections;

import org.apache.shiro.authc.AuthenticationException;
import org.apache.shiro.authc.AuthenticationInfo;
import org.apache.shiro.authc.AuthenticationToken;
import org.apache.shiro.authz.AuthorizationInfo;
import org.apache.shiro.authz.Permission;
import org.apache.shiro.authz.permission.AllPermission;
import org.apache.shiro.realm.AuthorizingRealm;
import org.apache.shiro.realm.Realm;
import org.apache.shiro.subject.PrincipalCollection;
import org.junit.Assert;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class ExceptionCatchingModularRealmAuthorizerTest
{
  private static final AuthorizingRealm BROKEN_REALM = new AuthorizingRealm()
  {
    @Override
    protected AuthenticationInfo doGetAuthenticationInfo(AuthenticationToken token) throws AuthenticationException {
      throw new RuntimeException("This realm only throws exceptions");
    }

    @Override
    protected AuthorizationInfo doGetAuthorizationInfo(PrincipalCollection principals) {
      throw new RuntimeException("This realm only throws exceptions");
    }
  };

  @Test
  public void ignoreRuntimeException() throws Exception {
    ExceptionCatchingModularRealmAuthorizer subject =
        new ExceptionCatchingModularRealmAuthorizer(Collections.<Realm>singleton(BROKEN_REALM));

    Permission permission = new AllPermission();

    Assert.assertFalse(subject.isPermitted(null, ""));
    Assert.assertFalse(subject.isPermitted(null, permission));
    Assert.assertFalse(subject.isPermitted(null, new String[]{""})[0]);
    Assert.assertFalse(subject.isPermitted(null, Collections.singletonList(permission))[0]);
  }
}
