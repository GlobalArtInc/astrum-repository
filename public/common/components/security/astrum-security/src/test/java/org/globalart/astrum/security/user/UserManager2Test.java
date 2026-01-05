
package org.globalart.astrum.security.user;

import java.util.HashMap;
import java.util.Map;
import java.util.Set;

import org.globalart.astrum.security.AbstractSecurityTest;
import org.globalart.astrum.security.config.PreconfiguredSecurityConfigurationSource;
import org.globalart.astrum.security.config.SecurityConfigurationSource;
import org.globalart.astrum.security.role.RoleIdentifier;
import org.globalart.astrum.security.user.UserManager2Test.UserManager2TestConfiguration;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Import;
import org.springframework.context.annotation.Primary;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasItem;
import static org.hamcrest.Matchers.hasKey;
import static org.hamcrest.Matchers.hasSize;
import static org.junit.jupiter.api.Assertions.assertEquals;

// FIXME: resolve with other UserManagerTest

/**
 * Tests for {@link UserManager}.
 */
@Import(UserManager2TestConfiguration.class)
class UserManager2Test
    extends AbstractSecurityTest
{
  static class UserManager2TestConfiguration
  {
    @Bean
    @Primary
    SecurityConfigurationSource securityConfigurationSource() {
      return new PreconfiguredSecurityConfigurationSource(UserManager2TestSecurity.securityModel());
    }
  }

  private UserManager underTest;

  @BeforeEach
  @Override
  protected void setUp() throws Exception {
    super.setUp();
    underTest = getUserManager();
  }

  @Test
  void testListUserIds() throws Exception {
    Set<String> userIds = underTest.listUserIds();
    assertThat(userIds, hasItem("test-user"));
    assertThat(userIds, hasItem("anonymous"));
    assertThat(userIds, hasItem("admin"));

    assertThat(userIds, hasSize(4));
  }

  @Test
  void testListUsers() throws Exception {
    Set<User> users = underTest.listUsers();
    Map<String, User> userMap = toUserMap(users);

    assertThat(userMap, hasKey("test-user"));
    assertThat(userMap, hasKey("anonymous"));
    assertThat(userMap, hasKey("admin"));

    assertEquals(4, users.size());
  }

  @Test
  void testGetUser() throws Exception {
    User testUser = underTest.getUser("test-user");

    assertEquals("Test User", testUser.getName());
    assertEquals("test-user", testUser.getUserId());
    assertEquals("test-user@example.org", testUser.getEmailAddress());

    // test roles
    Map<String, RoleIdentifier> roleMap = this.toRoleMap(testUser.getRoles());

    assertThat(roleMap, hasKey("role1"));
    assertThat(roleMap, hasKey("role2"));
    assertEquals(2, roleMap.size());
  }

  @Test
  void testGetUserWithEmptyRole() throws Exception {
    User testUser = underTest.getUser("test-user-with-empty-role");

    assertEquals("Test User With Empty Role", testUser.getName());
    assertEquals("test-user-with-empty-role", testUser.getUserId());
    assertEquals("test-user-with-empty-role@example.org", testUser.getEmailAddress());

    // test roles
    Map<String, RoleIdentifier> roleMap = this.toRoleMap(testUser.getRoles());

    assertThat(roleMap, hasKey("empty-role"));
    assertThat(roleMap, hasKey("role1"));
    assertThat(roleMap, hasKey("role2"));
    assertEquals(3, roleMap.size());
  }

  @Test
  void testSearchUser() throws Exception {
    Set<User> users = underTest.searchUsers(new UserSearchCriteria("test"));
    Map<String, User> userMap = toUserMap(users);

    assertThat(userMap, hasKey("test-user"));
    assertThat(userMap, hasKey("test-user-with-empty-role"));

    assertEquals(2, users.size());
  }

  private Map<String, RoleIdentifier> toRoleMap(final Set<RoleIdentifier> roles) {
    Map<String, RoleIdentifier> results = new HashMap<String, RoleIdentifier>();

    for (RoleIdentifier plexusRole : roles) {
      results.put(plexusRole.getRoleId(), plexusRole);
    }
    return results;
  }

  private static Map<String, User> toUserMap(final Set<User> users) {
    Map<String, User> results = new HashMap<String, User>();

    for (User plexusUser : users) {
      results.put(plexusUser.getUserId(), plexusUser);
    }
    return results;
  }
}
