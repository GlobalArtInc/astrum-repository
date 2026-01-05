
package org.globalart.astrum.security.realm;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.security.user.UserManager;

import org.apache.shiro.authc.AuthenticationException;
import org.apache.shiro.authc.AuthenticationInfo;
import org.apache.shiro.authc.AuthenticationToken;
import org.apache.shiro.authc.SimpleAuthenticationInfo;
import org.apache.shiro.authc.UsernamePasswordToken;
import org.apache.shiro.realm.AuthenticatingRealm;
import org.globalart.astrum.common.Description;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

@Component
@Qualifier("MockRealmA")
@Description("MockRealmA")
public class MockRealmA
    extends AuthenticatingRealm
{
  @Autowired
  public MockRealmA(@Qualifier("MockUserManagerA") UserManager userManager) {
    this.setAuthenticationTokenClass(UsernamePasswordToken.class);
  }

  @Override
  protected AuthenticationInfo doGetAuthenticationInfo(AuthenticationToken token) throws AuthenticationException {
    // only allow jcoder/jcoder

    UsernamePasswordToken userpass = (UsernamePasswordToken) token;
    if ("jcoder".equals(userpass.getUsername()) && "jcoder".equals(new String(userpass.getPassword()))) {
      return new SimpleAuthenticationInfo(userpass.getUsername(), new String(userpass.getPassword()), this.getName());
    }

    return null;
  }

  @Override
  public String getName() {
    return "MockRealmA";
  }
}
