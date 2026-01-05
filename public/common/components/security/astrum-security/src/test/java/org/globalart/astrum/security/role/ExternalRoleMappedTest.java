
package org.globalart.astrum.security.role;

import java.util.Collections;
import java.util.HashMap;
import java.util.Map;

import org.globalart.astrum.security.AbstractSecurityTest;
import org.globalart.astrum.security.AbstractSecurityTest.BaseSecurityConfiguration;
import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.internal.AuthorizingRealmImpl;
import org.globalart.astrum.security.privilege.Privilege;
import org.globalart.astrum.security.privilege.WildcardPrivilegeDescriptor;
import org.globalart.astrum.security.realm.MockRealm;
import org.globalart.astrum.security.realm.RealmManager;
import org.globalart.astrum.security.role.ExternalRoleMappedTest.ExternalRoleMappedTestConfiguration;
import org.globalart.astrum.security.user.MockUserManager;
import org.globalart.astrum.security.user.NoOpEventManager;
import org.globalart.astrum.security.user.UserManager;

import com.google.common.collect.ImmutableList;
import org.apache.shiro.authz.AuthorizationException;
import org.apache.shiro.subject.PrincipalCollection;
import org.apache.shiro.subject.SimplePrincipalCollection;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Import;
import org.springframework.context.annotation.Primary;
import org.springframework.test.context.TestPropertySource;

import static org.junit.jupiter.api.Assertions.fail;

@Import({BaseSecurityConfiguration.class, ExternalRoleMappedTestConfiguration.class})
@TestPropertySource(properties = {"astrum.security.principal.permissions.cache.enabled=false"})
public class ExternalRoleMappedTest
    extends AbstractSecurityTest
{

  static class ExternalRoleMappedTestConfiguration
  {
    @Bean
    public UserManager userManager() {
      return new MockUserManager(new NoOpEventManager());
    }

    @Primary
    @Bean
    public MockRealm mockRealm(@Qualifier("Mock") final UserManager userManager) {
      return new MockRealm(userManager);
    }
  }

  @Test
  void testUserHasPermissionFromExternalRole() throws Exception {
    SecuritySystem securitySystem = this.lookup(SecuritySystem.class);

    Map<String, String> properties = new HashMap<String, String>();
    properties.put(WildcardPrivilegeDescriptor.P_PATTERN, "permissionOne:read");

    securitySystem.getAuthorizationManager("default")
        .addPrivilege(new Privilege(
            "randomId",
            "permissionOne",
            "permissionOne",
            WildcardPrivilegeDescriptor.TYPE,
            properties,
            false));

    securitySystem.getAuthorizationManager("default")
        .addRole(new Role("mockrole1", "mockrole1", "mockrole1",
            "default", false, null,
            Collections.singleton("randomId")));

    // add MockRealm to config
    RealmManager realmManager = lookup(RealmManager.class);
    realmManager.setConfiguredRealmIds(ImmutableList.of("Mock", AuthorizingRealmImpl.NAME));

    // jcohen has the role mockrole1, there is also test role with the same ID, which means jcohen automaticly has
    // this test role

    PrincipalCollection jcohen = new SimplePrincipalCollection("jcohen", MockRealm.NAME);

    try {
      securitySystem.checkPermission(jcohen, "permissionOne:invalid");
      fail("Expected AuthorizationException");
    }
    catch (AuthorizationException e) {
      // expected
    }

    securitySystem.checkPermission(jcohen, "permissionOne:read"); // throws on error, so this is all we need to do
  }
}
