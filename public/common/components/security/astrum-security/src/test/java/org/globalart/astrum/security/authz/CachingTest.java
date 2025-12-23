
package org.globalart.astrum.security.authz;

import org.globalart.astrum.security.AbstractSecurityTest;
import org.globalart.astrum.security.AbstractSecurityTest.BaseSecurityConfiguration;
import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.realm.MockRealmB;
import org.globalart.astrum.security.user.User;

import org.apache.shiro.realm.Realm;
import org.apache.shiro.subject.SimplePrincipalCollection;
import org.junit.jupiter.api.Test;
import org.springframework.context.annotation.Import;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

@Import(BaseSecurityConfiguration.class)
class CachingTest
    extends AbstractSecurityTest
{
  @Test
  void testCacheClearing() throws Exception {
    SecuritySystem securitySystem = this.lookup(SecuritySystem.class);

    MockRealmB mockRealmB = (MockRealmB) this.lookup(Realm.class, "MockRealmB");

    // cache should be empty to start
    assertTrue(mockRealmB.getAuthorizationCache().keys().isEmpty());

    assertTrue(securitySystem.isPermitted(
        new SimplePrincipalCollection("jcool", mockRealmB.getName()), "test:heHasIt"));

    // now something will be in the cache, just make sure
    assertFalse(mockRealmB.getAuthorizationCache().keys().isEmpty());

    // now if we update a user the cache should be cleared
    User user = securitySystem.getUser("bburton", "MockUserManagerB");
    // different user, doesn't matter, in the future we should get a little more fine grained
    securitySystem.updateUser(user);

    // empty again
    assertTrue(mockRealmB.getAuthorizationCache().keys().isEmpty());
  }
}
