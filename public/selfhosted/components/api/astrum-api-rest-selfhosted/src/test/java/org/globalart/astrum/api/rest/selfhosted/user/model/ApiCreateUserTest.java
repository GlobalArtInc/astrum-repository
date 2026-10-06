
package org.globalart.astrum.api.rest.selfhosted.user.model;

import java.util.Collections;
import java.util.HashSet;
import java.util.Set;

import org.globalart.astrum.security.internal.rest.ApiUserStatus;
import org.globalart.astrum.security.user.User;
import org.globalart.astrum.security.user.UserManager;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.empty;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;

public class ApiCreateUserTest
{
  @Test
  public void testToUser_withNullRoles() {
    ApiCreateUser apiCreateUser = new ApiCreateUser(
        "testuser",
        "Test",
        "User",
        "test@example.com",
        "password123",
        ApiUserStatus.active,
        null);

    User user = apiCreateUser.toUser();

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
    ApiCreateUser apiCreateUser = new ApiCreateUser(
        "testuser",
        "Test",
        "User",
        "test@example.com",
        "password123",
        ApiUserStatus.active,
        Collections.emptySet());

    User user = apiCreateUser.toUser();

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

    ApiCreateUser apiCreateUser = new ApiCreateUser(
        "testuser",
        "Test",
        "User",
        "test@example.com",
        "password123",
        ApiUserStatus.active,
        roles);

    User user = apiCreateUser.toUser();

    assertThat(user, is(notNullValue()));
    assertThat(user.getUserId(), is("testuser"));
    assertThat(user.getRoles(), is(notNullValue()));
    assertThat(user.getRoles(), hasSize(2));
  }

  @Test
  public void testSetRoles_withNull() {
    ApiCreateUser apiCreateUser = new ApiCreateUser(
        "testuser",
        "Test",
        "User",
        "test@example.com",
        "password123",
        ApiUserStatus.active,
        new HashSet<>());

    apiCreateUser.setRoles(null);
    User user = apiCreateUser.toUser();

    assertThat(user, is(notNullValue()));
    assertThat(user.getRoles(), is(notNullValue()));
    assertThat(user.getRoles(), is(empty()));
  }
}
