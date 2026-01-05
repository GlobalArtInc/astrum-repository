
package org.globalart.astrum.security.user;

import java.util.HashSet;
import java.util.Set;

import org.globalart.astrum.security.AbstractSecurityTest;
import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.config.PreconfiguredSecurityConfigurationSource;
import org.globalart.astrum.security.config.SecurityConfigurationSource;
import org.globalart.astrum.security.role.RoleIdentifier;
import org.globalart.astrum.security.user.MissingRoleUserManagerTest.MissingRoleUserManagerTestConfiguration;

import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Import;
import org.springframework.context.annotation.Primary;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;

@Import(MissingRoleUserManagerTestConfiguration.class)
public class MissingRoleUserManagerTest
    extends AbstractSecurityTest
{
  static class MissingRoleUserManagerTestConfiguration
  {
    @Qualifier("default")
    @Primary
    @Bean
    SecurityConfigurationSource securityConfigurationSource() {
      return new PreconfiguredSecurityConfigurationSource(MissingRoleUserManagerTestSecurity.securityModel());
    }
  }

  @Test
  void testInvalidRoleMapping() throws Exception {
    SecuritySystem userManager = getSecuritySystem();

    User user = userManager.getUser("jcoder");
    assertNotNull(user);

    Set<String> roleIds = new HashSet<String>();
    for (RoleIdentifier role : user.getRoles()) {
      assertNotNull(role, "User has null role.");
      roleIds.add(role.getRoleId());
    }
    assertFalse(roleIds.contains("INVALID-ROLE-BLA-BLA"));
  }
}
