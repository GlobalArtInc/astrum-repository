
package org.globalart.astrum.security.internal.rest;

import java.util.Collections;
import java.util.HashSet;
import java.util.Set;

import org.globalart.astrum.security.user.User;
import org.globalart.astrum.security.user.UserManager;

import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.empty;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;

@RunWith(MockitoJUnitRunner.Silent.class)
public class ApiUserTest
{
  @Test
  public void testToUser_withNullRoles() {
    ApiUser apiUser = new ApiUser(
        "testuser",
        "Test",
        "User",
        "test@example.com",
        UserManager.DEFAULT_SOURCE,
        ApiUserStatus.active,
        false,
        null,
        null);

    User user = apiUser.toUser();

    assertThat(user, is(notNullValue()));
    assertThat(user.getUserId(), is("testuser"));
    assertThat(user.getFirstName(), is("Test"));
    assertThat(user.getLastName(), is("User"));
    assertThat(user.getEmailAddress(), is("test@example.com"));
    assertThat(user.getSource(), is(UserManager.DEFAULT_SOURCE));
    assertThat(user.getRoles(), is(notNullValue()));
    assertThat(user.getRoles(), is(empty()));
  }

  @Test
  public void testToUser_withEmptyRoles() {
    ApiUser apiUser = new ApiUser(
        "testuser",
        "Test",
        "User",
        "test@example.com",
        UserManager.DEFAULT_SOURCE,
        ApiUserStatus.active,
        false,
        Collections.emptySet(),
        null);

    User user = apiUser.toUser();

    assertThat(user, is(notNullValue()));
    assertThat(user.getUserId(), is("testuser"));
    assertThat(user.getRoles(), is(notNullValue()));
    assertThat(user.getRoles(), is(empty()));
  }

  @Test
  public void testToUser_withRoles() {
    Set<String> roles = new HashSet<>();
    roles.add("as-admin");
    roles.add("as-developer");

    ApiUser apiUser = new ApiUser(
        "testuser",
        "Test",
        "User",
        "test@example.com",
        UserManager.DEFAULT_SOURCE,
        ApiUserStatus.active,
        false,
        roles,
        null);

    User user = apiUser.toUser();

    assertThat(user, is(notNullValue()));
    assertThat(user.getUserId(), is("testuser"));
    assertThat(user.getRoles(), is(notNullValue()));
    assertThat(user.getRoles(), hasSize(2));
  }

  @Test
  public void testToUser_withExternalRoles() {
    Set<String> roles = new HashSet<>();
    roles.add("as-admin");

    Set<String> externalRoles = new HashSet<>();
    externalRoles.add("ldap-group1");
    externalRoles.add("ldap-group2");

    ApiUser apiUser = new ApiUser(
        "testuser",
        "Test",
        "User",
        "test@example.com",
        "LDAP",
        ApiUserStatus.active,
        false,
        roles,
        externalRoles);

    User user = apiUser.toUser();

    assertThat(user, is(notNullValue()));
    assertThat(user.getUserId(), is("testuser"));
    assertThat(user.getRoles(), is(notNullValue()));
    assertThat(user.getRoles(), hasSize(3));
  }

  @Test
  public void testSetRoles_withNull() {
    ApiUser apiUser = new ApiUser(
        "testuser",
        "Test",
        "User",
        "test@example.com",
        UserManager.DEFAULT_SOURCE,
        ApiUserStatus.active,
        false,
        new HashSet<>(),
        null);

    apiUser.setRoles(null);
    User user = apiUser.toUser();

    assertThat(user, is(notNullValue()));
    assertThat(user.getRoles(), is(notNullValue()));
    assertThat(user.getRoles(), is(empty()));
  }
}
