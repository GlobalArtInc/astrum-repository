
package org.globalart.astrum.security.user;

import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

import org.globalart.astrum.security.AbstractSecurityTest;
import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.config.CUser;
import org.globalart.astrum.security.config.CUserRoleMapping;
import org.globalart.astrum.security.config.PreconfiguredSecurityConfigurationSource;
import org.globalart.astrum.security.config.SecurityConfiguration;
import org.globalart.astrum.security.config.SecurityConfigurationManager;
import org.globalart.astrum.security.config.SecurityConfigurationSource;
import org.globalart.astrum.security.config.memory.MemoryCUser;
import org.globalart.astrum.security.role.RoleIdentifier;
import org.globalart.astrum.security.user.UserManagerTest.UserManagerTestSecurityConfiguration;

import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Import;
import org.springframework.context.annotation.Primary;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.is;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.junit.jupiter.api.Assertions.fail;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.when;

// FIXME: resolve with other UserManager2Test

@Import(UserManagerTestSecurityConfiguration.class)
class UserManagerTest
    extends AbstractSecurityTest
{

  @Override
  @BeforeEach
  protected void setUp() throws Exception {
    super.setUp();
  }

  @AfterEach
  void restoreTestUser() {
    CUser originalUser = new MemoryCUser()
        .withId("test-user")
        .withPassword("b2a0e378437817cebdf753d7dff3dd75483af9e0")
        .withFirstName("Test")
        .withLastName("User")
        .withStatus("active")
        .withEmail("test-user@example.org");
    Set<String> roles = Set.of("role1", "role2");

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
  void testGetUser() throws Exception {
    UserManager userManager = this.getUserManager();

    User user = userManager.getUser("test-user");

    assertEquals("test-user", user.getUserId());
    assertEquals("test-user@example.org", user.getEmailAddress());
    assertEquals("Test User", user.getName());
    // not exposed anymore
    // assertEquals( user.getPassword(), "b2a0e378437817cebdf753d7dff3dd75483af9e0" );
    assertEquals("active", user.getStatus().name());

    List<String> roleIds = getRoleIds(user);

    assertTrue(roleIds.contains("role1"));
    assertTrue(roleIds.contains("role2"));
    assertEquals(2, roleIds.size());
  }

  @Test
  void testAddUser() throws Exception {
    when(passwordService.encryptPassword(eq("my-password")))
        .thenReturn("$PBKDF2WithHmacSHA256$mocking$mocked-hashed-password");
    UserManager userManager = this.getUserManager();
    User user = new User();
    user.setUserId("testCreateUser");
    user.setName(user.getUserId() + "-name");
    user.setSource(user.getUserId() + "default");
    user.setEmailAddress("email@email");
    user.setStatus(UserStatus.active);
    user.addRole(new RoleIdentifier("default", "role1"));
    user.addRole(new RoleIdentifier("default", "role3"));

    userManager.addUser(user, "my-password");

    SecurityConfigurationManager config = this.getConfigurationManager();

    CUser secUser = config.readUser(user.getUserId());
    assertEquals(secUser.getId(), user.getUserId());
    assertEquals(secUser.getEmail(), user.getEmailAddress());
    assertEquals(secUser.getFirstName(), user.getFirstName());
    assertEquals(secUser.getLastName(), user.getLastName());
    assertEquals(secUser.getPassword(), "$PBKDF2WithHmacSHA256$mocking$mocked-hashed-password");
    assertEquals(secUser.getStatus(), user.getStatus().name());

    CUserRoleMapping roleMapping = config.readUserRoleMapping("testCreateUser", "default");

    assertTrue(roleMapping.getRoles().contains("role1"));
    assertTrue(roleMapping.getRoles().contains("role3"));
    assertEquals(2, roleMapping.getRoles().size());
  }

  @Test
  void testSupportsWrite() {
    assertTrue(this.getUserManager().supportsWrite());
  }

  @Test
  void testChangePassword() throws Exception {
    when(passwordService.encryptPassword(eq("new-user-password")))
        .thenReturn("$PBKDF2WithHmacSHA1$mocking$new-user-password-hashed");
    UserManager userManager = this.getUserManager();
    userManager.changePassword("test-user", "new-user-password");

    CUser user = this.getConfigurationManager().readUser("test-user");
    assertThat(user.getPassword().startsWith("$PBKDF2WithHmacSHA1"), is(true));
  }

  @Test
  void testUpdateUser() throws Exception {
    UserManager userManager = this.getUserManager();

    User user = userManager.getUser("test-user");

    user.setName("new Name");
    user.setEmailAddress("newemail@foo");

    Set<RoleIdentifier> roles = new HashSet<>();
    roles.add(new RoleIdentifier("default", "role3"));
    user.setRoles(roles);
    userManager.updateUser(user);

    SecurityConfigurationManager config = this.getConfigurationManager();

    CUser secUser = config.readUser(user.getUserId());
    assertEquals(secUser.getId(), user.getUserId());
    assertEquals(secUser.getEmail(), user.getEmailAddress());
    assertEquals(secUser.getFirstName(), user.getFirstName());
    assertEquals(secUser.getLastName(), user.getLastName());
    assertEquals("b2a0e378437817cebdf753d7dff3dd75483af9e0", secUser.getPassword());

    assertEquals(secUser.getStatus(), user.getStatus().name());

    CUserRoleMapping roleMapping = config.readUserRoleMapping("test-user", "default");

    assertTrue(roleMapping.getRoles().contains("role3"));
    assertEquals(1, roleMapping.getRoles().size(), "roles: " + roleMapping.getRoles());
  }

  @Test
  void testDeleteUser() throws Exception {
    UserManager userManager = this.getUserManager();
    try {
      userManager.deleteUser("INVALID-USERNAME");
      fail("Expected UserNotFoundException");
    }
    catch (UserNotFoundException e) {
      // expected
    }

    // this one will work
    userManager.deleteUser("test-user");

    // this one should fail
    try {
      userManager.deleteUser("test-user");
      fail("Expected UserNotFoundException");
    }
    catch (UserNotFoundException e) {
      // expected
    }

    try {
      userManager.getUser("test-user");
      fail("Expected UserNotFoundException");
    }
    catch (UserNotFoundException e) {
      // expected
    }

    try {
      this.getConfigurationManager().readUser("test-user");
      fail("Expected UserNotFoundException");
    }
    catch (UserNotFoundException e) {
      // expected
    }
  }

  @Test
  void testDeleteUserAndUserRoleMappings() throws Exception {
    String userId = "testDeleteUserAndUserRoleMappings";

    UserManager userManager = this.getUserManager();

    User user = new User();
    user.setUserId(userId);
    user.setName(user.getUserId() + "-name");
    user.setSource(user.getUserId() + "default");
    user.setEmailAddress("email@email");
    user.setStatus(UserStatus.active);
    user.addRole(new RoleIdentifier("default", "role1"));
    user.addRole(new RoleIdentifier("default", "role3"));

    userManager.addUser(user, "my-password");

    // now delete the user
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
  void testSetUsersRoles() throws Exception {
    SecuritySystem securitySystem = this.getSecuritySystem();

    Set<RoleIdentifier> roleIdentifiers = new HashSet<>();
    RoleIdentifier roleIdentifier = new RoleIdentifier("default", "role2");
    roleIdentifiers.add(roleIdentifier);

    securitySystem.setUsersRoles("admin", "default", roleIdentifiers);

    SecurityConfiguration securityModel = this.getSecurityConfiguration();

    boolean found = false;
    for (CUserRoleMapping roleMapping : securityModel.getUserRoleMappings()) {
      if (roleMapping.getUserId().equals("admin")) {
        found = true;

        assertThat(roleMapping.getRoles(), contains("role2"));
      }
    }

    assertTrue(found, "did not find admin user in role mapping");
  }

  private SecurityConfigurationManager getConfigurationManager() {
    return lookup(SecurityConfigurationManager.class);
  }

  private static List<String> getRoleIds(final User user) {
    List<String> roleIds = new ArrayList<>();

    for (RoleIdentifier role : user.getRoles()) {
      roleIds.add(role.getRoleId());
    }

    return roleIds;
  }

  protected static class UserManagerTestSecurityConfiguration
  {
    @Qualifier("default")
    @Primary
    @Bean
    public SecurityConfigurationSource securityConfigurationSource() {
      return new PreconfiguredSecurityConfigurationSource(UserManagerTestSecurity.securityModel());
    }
  }
}
