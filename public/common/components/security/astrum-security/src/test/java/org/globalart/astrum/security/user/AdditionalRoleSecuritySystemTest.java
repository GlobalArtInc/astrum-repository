
package org.globalart.astrum.security.user;

import java.util.HashMap;
import java.util.HashSet;
import java.util.Map;
import java.util.Set;

import org.globalart.astrum.security.AbstractSecurityTest;
import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.config.PreconfiguredSecurityConfigurationSource;
import org.globalart.astrum.security.config.SecurityConfigurationSource;
import org.globalart.astrum.security.role.RoleIdentifier;
import org.globalart.astrum.security.user.AdditionalRoleSecuritySystemTest.AdditionalRoleSecuritySystemTestSecurityConfiguration;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Import;
import org.springframework.context.annotation.Primary;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.aMapWithSize;
import static org.hamcrest.Matchers.empty;
import static org.hamcrest.Matchers.hasItems;
import static org.hamcrest.Matchers.hasSize;
import static org.junit.jupiter.api.Assertions.assertNotNull;

@Import(AdditionalRoleSecuritySystemTestSecurityConfiguration.class)
class AdditionalRoleSecuritySystemTest
    extends AbstractSecurityTest
{
  protected static class AdditionalRoleSecuritySystemTestSecurityConfiguration
  {
    @Qualifier("default")
    @Primary
    @Bean
    public SecurityConfigurationSource securityConfigurationSource() {
      return new PreconfiguredSecurityConfigurationSource(AdditionalRoleSecuritySystemTestSecurity.securityModel());
    }
  }

  private SecuritySystem securitySystem;

  @Override
  @BeforeEach
  public void setUp() throws Exception {
    super.setUp();

    securitySystem = getSecuritySystem();
  }

  @Test
  public void testListUsers() throws Exception {
    UserSearchCriteria criteria = new UserSearchCriteria(null, null, "MockUserManagerA");
    Set<User> users = securitySystem.searchUsers(criteria);

    Map<String, User> userMap = toUserMap(users);

    User user = userMap.get("jcoder");
    assertNotNull(user);

    // A,B,C,1
    Set<String> roleIds = toRoleIdSet(user.getRoles());
    assertThat(roleIds, hasItems("RoleA", "RoleB", "RoleC", "Role1"));
    assertThat(roleIds, hasSize(4));

    user = userMap.get("dknudsen");
    assertNotNull(user);
    assertThat(user.getRoles(), hasSize(1));

    // Role2
    roleIds = toRoleIdSet(user.getRoles());
    assertThat(roleIds, hasItems("Role2"));

    user = userMap.get("cdugas");
    assertNotNull(user);
    assertThat(user.getRoles(), hasSize(3));

    // A,B,1
    roleIds = toRoleIdSet(user.getRoles());
    assertThat(roleIds, hasItems("RoleA", "RoleB", "Role1"));

    user = userMap.get("pperalez");
    assertNotNull(user);
    assertThat(user.getRoles(), empty());
  }

  @Test
  public void testSearchEffectiveFalse() throws Exception {
    UserSearchCriteria criteria = new UserSearchCriteria();

    criteria.setUserId("pperalez");
    User user = searchForSingleUser(criteria, "pperalez", "MockUserManagerA");
    assertNotNull(user);

    criteria.setUserId("jcoder");
    user = searchForSingleUser(criteria, "jcoder", "MockUserManagerA");
    assertNotNull(user);

    // A,B,C,1
    Set<String> roleIds = toRoleIdSet(user.getRoles());
    assertThat(roleIds, hasItems("RoleA", "RoleB", "RoleC", "Role1"));
    assertThat(user.getRoles(), hasSize(4));

    criteria.setUserId("dknudsen");
    user = searchForSingleUser(criteria, "dknudsen", "MockUserManagerA");
    assertNotNull(user);
    assertThat(user.getRoles(), hasSize(1));

    // Role2
    roleIds = toRoleIdSet(user.getRoles());
    assertThat(roleIds, hasItems("Role2"));

    criteria.setUserId("cdugas");
    user = searchForSingleUser(criteria, "cdugas", "MockUserManagerA");
    assertNotNull(user);
    assertThat(user.getRoles(), hasSize(3));

    // A,B,1
    roleIds = toRoleIdSet(user.getRoles());
    assertThat(roleIds, hasItems("RoleA", "RoleB", "Role1"));
  }

  private User searchForSingleUser(
      final UserSearchCriteria criteria,
      final String userId,
      final String source) throws Exception
  {
    criteria.setSource(source);
    Set<User> users = securitySystem.searchUsers(criteria);

    System.out.println("Found users:");
    for (User user : users) {
      System.out.format("%s, source=%s%n", user, user.getSource());
    }

    Map<String, User> userMap = toUserMap(users);
    assertThat(userMap, aMapWithSize(1));

    return userMap.get(userId);
  }

  private Map<String, User> toUserMap(final Set<User> users) {
    HashMap<String, User> map = new HashMap<String, User>();
    for (User plexusUser : users) {
      map.put(plexusUser.getUserId(), plexusUser);
    }
    return map;
  }

  private Set<String> toRoleIdSet(final Set<RoleIdentifier> roles) {
    Set<String> roleIds = new HashSet<String>();
    for (RoleIdentifier role : roles) {
      roleIds.add(role.getRoleId());
    }
    return roleIds;
  }
}
