
package org.globalart.astrum.security.user;

import java.util.HashSet;
import java.util.Set;

import org.globalart.astrum.security.AbstractSecurityTest;
import org.globalart.astrum.security.config.CUser;
import org.globalart.astrum.security.config.CUserRoleMapping;
import org.globalart.astrum.security.config.PreconfiguredSecurityConfigurationSource;
import org.globalart.astrum.security.config.SecurityConfigurationManager;
import org.globalart.astrum.security.config.SecurityConfigurationSource;
import org.globalart.astrum.security.internal.SecurityConfigurationManagerImpl;
import org.globalart.astrum.security.user.UserRoleMappingTest.UserRoleMappingTestConfiguration;

import org.junit.jupiter.api.Test;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Import;
import org.springframework.context.annotation.Primary;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

@Import(UserRoleMappingTestConfiguration.class)
class UserRoleMappingTest
    extends AbstractSecurityTest
{
  static class UserRoleMappingTestConfiguration
  {
    @Bean
    @Primary
    SecurityConfigurationSource securityConfigurationSource() {
      return new PreconfiguredSecurityConfigurationSource(UserRoleMappingTestSecurity.securityModel());
    }
  }

  SecurityConfigurationManager getConfigManager() throws Exception {
    return this.lookup(SecurityConfigurationManagerImpl.class);
  }

  @Test
  void testGetUser() throws Exception {
    SecurityConfigurationManager config = this.getConfigManager();

    CUser user = config.readUser("test-user");
    assertEquals(user.getId(), "test-user");
    assertEquals(user.getEmail(), "test-user@example.org");
    assertEquals(user.getFirstName(), "Test");
    assertEquals(user.getLastName(), "User");
    assertEquals(user.getPassword(), "b2a0e378437817cebdf753d7dff3dd75483af9e0");
    assertEquals(user.getStatus(), "active");

    CUserRoleMapping mapping = config.readUserRoleMapping("test-user", "default");

    assertTrue(mapping.getRoles().contains("role1"));
    assertTrue(mapping.getRoles().contains("role2"));
    assertEquals(2, mapping.getRoles().size());
  }

  @Test
  void testGetUserWithEmptyRole() throws Exception {
    SecurityConfigurationManager config = this.getConfigManager();

    CUser user = config.readUser("test-user-with-empty-role");
    assertEquals(user.getId(), "test-user-with-empty-role");
    assertEquals(user.getEmail(), "test-user-with-empty-role@example.org");
    assertEquals(user.getFirstName(), "Test");
    assertEquals(user.getLastName(), "User With Empty Role");
    assertEquals(user.getPassword(), "b2a0e378437817cebdf753d7dff3dd75483af9e0");
    assertEquals(user.getStatus(), "active");

    CUserRoleMapping mapping = config.readUserRoleMapping("test-user-with-empty-role", "default");

    assertTrue(mapping.getRoles().contains("empty-role"));
    assertTrue(mapping.getRoles().contains("role1"));
    assertTrue(mapping.getRoles().contains("role2"));
    assertEquals(3, mapping.getRoles().size());

    // try to update empty role
    config.updateUserRoleMapping(mapping);

    // make sure we still have the role mappings
    mapping = config.readUserRoleMapping("test-user-with-empty-role", "default");

    assertTrue(mapping.getRoles().contains("empty-role"));
    assertTrue(mapping.getRoles().contains("role1"));
    assertTrue(mapping.getRoles().contains("role2"));
    assertEquals(3, mapping.getRoles().size());
  }

  @Test
  void testUpdateUsersRoles() throws Exception {
    SecurityConfigurationManager config = this.getConfigManager();

    // make sure we have exactly 4 user role mappings
    assertEquals(5, config.listUserRoleMappings().size());

    // get the test-user and add a role
    CUser user = config.readUser("test-user");

    CUserRoleMapping roleMapping = config.readUserRoleMapping("test-user", "default");
    Set<String> roles = roleMapping.getRoles();
    roles.add("role3");

    // update the user
    config.updateUser(user, new HashSet<String>(roles));

    // make sure we have exactly 4 user role mappings
    assertEquals(5, config.listUserRoleMappings().size());
  }
}
