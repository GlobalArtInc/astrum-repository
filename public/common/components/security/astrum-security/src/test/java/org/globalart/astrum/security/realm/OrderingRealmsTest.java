
package org.globalart.astrum.security.realm;

import org.globalart.astrum.security.AbstractSecurityTest;
import org.globalart.astrum.security.AbstractSecurityTest.BaseSecurityConfiguration;
import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.user.User;

import com.google.common.collect.ImmutableList;
import org.junit.jupiter.api.Test;
import org.springframework.context.annotation.Import;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;

@Import(BaseSecurityConfiguration.class)
public class OrderingRealmsTest
    extends AbstractSecurityTest
{
  @Test
  void testOrderedGetUser() throws Exception {
    SecuritySystem securitySystem = this.lookup(SecuritySystem.class);
    RealmManager realmManager = lookup(RealmManager.class);
    realmManager.setConfiguredRealmIds(ImmutableList.of("MockRealmA", "MockRealmB"));

    User jcoder = securitySystem.getUser("jcoder");
    assertNotNull(jcoder);

    // make sure jcoder is from MockUserManagerA
    assertEquals("MockUserManagerA", jcoder.getSource());

    // now change the order
    realmManager.setConfiguredRealmIds(ImmutableList.of("MockRealmB", "MockRealmA"));

    jcoder = securitySystem.getUser("jcoder");
    assertNotNull(jcoder);

    // make sure jcoder is from MockUserManagerA
    assertEquals("MockUserManagerB", jcoder.getSource());
  }
}
