
package org.globalart.astrum.security.user;

import java.util.Set;

import org.globalart.astrum.security.AbstractSecurityTest;
import org.globalart.astrum.security.config.CUser;
import org.globalart.astrum.security.config.CUserRoleMapping;
import org.globalart.astrum.security.config.PreconfiguredSecurityConfigurationSource;
import org.globalart.astrum.security.config.SecurityConfiguration;
import org.globalart.astrum.security.config.SecurityConfigurationSource;
import org.globalart.astrum.security.config.memory.MemoryCUser;
import org.globalart.astrum.security.role.RoleIdentifier;
import org.globalart.astrum.security.user.EmptyRoleManagementTest.EmptyRoleManagementTestSecurityConfiguration;

import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Import;
import org.springframework.context.annotation.Primary;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.junit.jupiter.api.Assertions.fail;

@Import(EmptyRoleManagementTestSecurityConfiguration.class)
class EmptyRoleManagementTest
    extends AbstractSecurityTest
{
  protected static class EmptyRoleManagementTestSecurityConfiguration
  {
    @Qualifier("default")
    @Primary
    @Bean
    SecurityConfigurationSource securityConfigurationSource() {
      return new PreconfiguredSecurityConfigurationSource(EmptyRoleManagementTestSecurity.securityModel());
    }
  }

  @AfterEach
  void restoreUserWithEmptyRole() {
    CUser originalUser = new MemoryCUser()
        .withId("test-user-with-empty-role")
        .withPassword("b2a0e378437817cebdf753d7dff3dd75483af9e0")
        .withFirstName("Test User With Empty Role")
        .withStatus("active")
        .withEmail("test-user-with-empty-role@example.org");
    Set<String> roles = Set.of("empty-role", "role1", "role2");

    SecurityConfiguration securityModel = this.getSecurityConfiguration();
    try {
      securityModel.updateUser(originalUser, roles);
    }
    catch (UserNotFoundException e) {
      // user does not exist, so we need to restore it
      securityModel.addUser(originalUser, roles);
    }
  }

  @Test
  void testDeleteUserWithEmptyRole() throws Exception {
    String userId = "test-user-with-empty-role";

    UserManager userManager = this.getUserManager();
    userManager.deleteUser(userId);

    SecurityConfiguration securityModel = this.getSecurityConfiguration();

    for (CUser tmpUser : securityModel.getUsers()) {
      if (userId.equals(tmpUser.getId())) {
        fail("User " + userId + " was not removed.");
      }
    }

    for (CUserRoleMapping userRoleMapping : securityModel.getUserRoleMappings()) {
      if (userId.equals(userRoleMapping.getUserId()) && "default".equals(userRoleMapping.getSource())) {
        fail("User Role Mapping was not deleted when user: " + userId + " was removed.");
      }
    }
  }

  @Test
  void testDeleteEmptyRoleFromUser() throws Exception {
    String userId = "test-user-with-empty-role";
    String roleId = "empty-role";

    RoleIdentifier emptyRole = new RoleIdentifier("default", roleId);

    UserManager userManager = this.getUserManager();
    User user = userManager.getUser(userId);

    assertEquals(3, user.getRoles().size());
    assertTrue(user.getRoles().contains(emptyRole));

    user.removeRole(emptyRole);

    assertEquals(2, user.getRoles().size());
    assertFalse(user.getRoles().contains(emptyRole));

    userManager.updateUser(user);

    SecurityConfiguration securityModel = this.getSecurityConfiguration();
    for (CUserRoleMapping userRoleMapping : securityModel.getUserRoleMappings()) {
      if (userId.equals(userRoleMapping.getUserId()) && "default".equals(userRoleMapping.getSource())) {
        Set<String> configuredRoles = userRoleMapping.getRoles();
        assertEquals(2, configuredRoles.size());
        assertFalse(configuredRoles.contains(roleId));
      }
    }
  }

  @Test
  void testUpdateUser() throws Exception {
    String userId = "test-user-with-empty-role";

    UserManager userManager = this.getUserManager();
    User user = userManager.getUser(userId);

    String value = "value";
    user.setEmailAddress(String.format("%s@%s", value, value));
    user.setFirstName(value);
    user.setLastName(value);

    userManager.updateUser(user);

    SecurityConfiguration securityModel = this.getSecurityConfiguration();

    boolean found = false;
    for (CUser tmpUser : securityModel.getUsers()) {
      if (userId.equals(tmpUser.getId())) {
        assertEquals(String.format("%s@%s", value, value), user.getEmailAddress());
        assertEquals(value, user.getFirstName());
        assertEquals(value, user.getLastName());
        found = true;
      }
    }
    assertTrue(found, "user not found");

    found = false;
    for (CUserRoleMapping userRoleMapping : securityModel.getUserRoleMappings()) {
      if (userId.equals(userRoleMapping.getUserId()) && "default".equals(userRoleMapping.getSource())) {
        assertEquals(3, userRoleMapping.getRoles().size());
        found = true;
      }
    }

    assertTrue(found, "userRoleMapping not found");
  }

  @Test
  void testDeleteOtherRoleFromUser() throws Exception {
    String userId = "test-user-with-empty-role";
    String roleId = "role1";

    RoleIdentifier emptyRole = new RoleIdentifier("default", roleId);

    UserManager userManager = this.getUserManager();
    User user = userManager.getUser(userId);

    assertEquals(3, user.getRoles().size());
    assertTrue(user.getRoles().contains(emptyRole));

    user.removeRole(emptyRole);

    assertEquals(2, user.getRoles().size());
    assertFalse(user.getRoles().contains(emptyRole));

    userManager.updateUser(user);

    SecurityConfiguration securityModel = this.getSecurityConfiguration();
    for (CUserRoleMapping userRoleMapping : securityModel.getUserRoleMappings()) {
      if (userId.equals(userRoleMapping.getUserId()) && "default".equals(userRoleMapping.getSource())) {
        Set<String> configuredRoles = userRoleMapping.getRoles();
        assertEquals(2, configuredRoles.size());
        assertFalse(configuredRoles.contains(roleId));
      }
    }
  }
}
