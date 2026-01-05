
package org.globalart.astrum.security.user;

import java.util.HashMap;
import java.util.Map;
import java.util.Set;

import org.globalart.astrum.security.AbstractSecurityTest;
import org.globalart.astrum.security.AbstractSecurityTest.BaseSecurityConfiguration;
import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.realm.RealmManager;
import org.globalart.astrum.security.user.UserManagementTest.UserManagementTestConfiguration;

import com.google.common.collect.ImmutableList;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Import;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

@Import({BaseSecurityConfiguration.class, UserManagementTestConfiguration.class})
class UserManagementTest
    extends AbstractSecurityTest
{
  static class UserManagementTestConfiguration
  {
    @Bean
    UserManager userManager() {
      return new MockUserManager(new NoOpEventManager());
    }
  }

  private SecuritySystem securitySystem;

  @BeforeEach
  @Override
  protected void setUp() throws Exception {
    super.setUp();

    securitySystem = getSecuritySystem();

    RealmManager realmManager = lookup(RealmManager.class);
    realmManager.setConfiguredRealmIds(ImmutableList.of("MockRealmA", "MockRealmB"));
  }

  @Test
  void testAllUsers() throws Exception {
    Set<User> users = securitySystem.listUsers();
    assertFalse(users.isEmpty());

    // put users in map for easy search
    Map<String, User> userMap = getMapFromSet(users);

    // now check all of the users
    assertTrue(userMap.containsKey("jcoder"));
    assertTrue(userMap.containsKey("cdugas"));
    assertTrue(userMap.containsKey("pperalez"));
    assertTrue(userMap.containsKey("dknudsen"));
    assertTrue(userMap.containsKey("anonymous-user"));

    assertTrue(userMap.containsKey("bburton"));
    assertTrue(userMap.containsKey("jblevins"));
    assertTrue(userMap.containsKey("ksimmons"));
    assertTrue(userMap.containsKey("fdahmen"));
    assertTrue(userMap.containsKey("jcodar"));

    // FIXME: This is a pretty fragile assertion
    assertEquals(15, users.size());

    // we just need to check to make sure there are 2 jcoders with the correct source
    verify2Jcoders(users);
  }

  @Test
  void testSearchWithCriteria() throws Exception {
    UserSearchCriteria criteria = new UserSearchCriteria();

    criteria.setUserId("pperalez");
    Set<User> users = securitySystem.searchUsers(criteria);
    assertEquals(1, users.size());
    assertEquals("pperalez", users.iterator().next().getUserId());

    criteria.setUserId("ppera");
    users = securitySystem.searchUsers(criteria);
    assertEquals(1, users.size());
    assertEquals("pperalez", users.iterator().next().getUserId());

    criteria.setUserId("ppera");
    criteria.setSource("MockUserManagerB");
    users = securitySystem.searchUsers(criteria);
    assertEquals(0, users.size());

    criteria.setUserId("ksim");
    users = securitySystem.searchUsers(criteria);
    assertEquals(1, users.size());
    assertEquals("ksimmons", users.iterator().next().getUserId());

    criteria.setUserId("jcod");
    criteria.setSource(null);
    users = securitySystem.searchUsers(criteria);
    assertEquals(3, users.size());

    // put users in map for easy search
    Map<String, User> userMap = getMapFromSet(users);

    assertTrue(userMap.containsKey("jcodar"));

    // we just need to check to make sure there are 2 jcoders with the correct source (the counts are already
    // checked above)
    verify2Jcoders(users);
  }

  private static Map<String, User> getMapFromSet(final Set<User> users) {
    Map<String, User> userMap = new HashMap<String, User>();
    for (User user : users) {
      userMap.put(user.getUserId(), user);
    }
    return userMap;
  }

  private static void verify2Jcoders(final Set<User> users) {
    Map<String, User> jcoders = new HashMap<String, User>();
    for (User user : users) {
      if (user.getUserId().equals("jcoder")) {
        jcoders.put(user.getSource(), user);
      }
    }
    assertEquals(2, jcoders.size());
    assertTrue(jcoders.containsKey("MockUserManagerA"));
    assertTrue(jcoders.containsKey("MockUserManagerB"));
  }
}
