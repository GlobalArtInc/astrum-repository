
package org.globalart.astrum.security.internal;

import java.util.List;

import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.TestAnonymousConfiguration;
import org.globalart.astrum.security.anonymous.AnonymousConfiguration;
import org.globalart.astrum.security.anonymous.AnonymousManager;
import org.globalart.astrum.security.authz.AuthorizationManager;
import org.globalart.astrum.security.authz.NoSuchAuthorizationManagerException;
import org.globalart.astrum.security.role.Role;
import org.globalart.astrum.security.role.RoleIdentifier;
import org.globalart.astrum.security.user.NoSuchUserManagerException;
import org.globalart.astrum.security.user.User;
import org.globalart.astrum.security.user.UserManager;
import org.globalart.astrum.security.user.UserStatus;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.ArgumentCaptor;
import org.mockito.Captor;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.is;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * @since 3.0
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class SecurityApiImplTest
{
  @Mock
  private AnonymousManager anonymousManager;

  @Mock
  private SecuritySystem securitySystem;

  @Mock
  private AuthorizationManager authorizationManager;

  @Captor
  private ArgumentCaptor<User> userCaptor;

  private TestAnonymousConfiguration configuration = new TestAnonymousConfiguration();

  @InjectMocks
  private SecurityApiImpl api;

  @Before
  public void setup() throws NoSuchAuthorizationManagerException {
    when(securitySystem.getAuthorizationManager(UserManager.DEFAULT_SOURCE)).thenReturn(authorizationManager);
    when(anonymousManager.getConfiguration()).thenReturn(configuration);
  }

  @Test
  public void testSetAnonymousAccess() {
    configuration.setEnabled(true);

    AnonymousConfiguration updatedConfiguration = api.setAnonymousAccess(false);

    assertFalse(updatedConfiguration.isEnabled());
    verify(anonymousManager).getConfiguration();
    verify(anonymousManager).setConfiguration(any());
  }

  /*
   * No save is made when configured and anonymous settings already match
   */
  @Test
  public void testSetAnonymousAccess_unchanged() {
    configuration.setEnabled(true);
    when(anonymousManager.isConfigured()).thenReturn(true);

    AnonymousConfiguration updatedConfiguration = api.setAnonymousAccess(true);

    assertTrue(updatedConfiguration.isEnabled());
    verify(anonymousManager).getConfiguration();
    verify(anonymousManager, never()).setConfiguration(any());
  }

  /*
   * One save is made when unconfigured and anonymous settings already match
   */
  @Test
  public void testSetAnonymousAccess_unconfigured() {
    when(anonymousManager.isConfigured()).thenReturn(false);

    AnonymousConfiguration updatedConfiguration = api.setAnonymousAccess(false);

    assertFalse(updatedConfiguration.isEnabled());

    verify(anonymousManager).getConfiguration();
    verify(anonymousManager).setConfiguration(any());
  }

  @Test
  public void testAddUser() throws NoSuchUserManagerException {
    when(securitySystem.addUser(any(), eq("pass"))).thenAnswer(i -> i.getArguments()[0]);

    User user = api.addUser("foo", "bar", "baz", "foo@bar.com", true, "pass", List.of("roleId"));

    verify(securitySystem).addUser(any(), eq("pass"));

    assertThat(user.getUserId(), is("foo"));
    assertThat(user.getSource(), is(UserManager.DEFAULT_SOURCE));
    assertThat(user.getFirstName(), is("bar"));
    assertThat(user.getLastName(), is("baz"));
    assertThat(user.getEmailAddress(), is("foo@bar.com"));
    assertThat(user.getStatus(), is(UserStatus.active));
    assertThat(user.getRoles(), contains(new RoleIdentifier(UserManager.DEFAULT_SOURCE, "roleId")));
  }

  @Test
  public void testAddRole() throws NoSuchAuthorizationManagerException {
    when(authorizationManager.addRole(any())).thenAnswer(i -> i.getArguments()[0]);

    Role role = api.addRole("foo", "bar", "baz", List.of("priv"), List.of("role"));

    verify(securitySystem).getAuthorizationManager(any());
    verify(authorizationManager).addRole(any());

    assertThat(role.getRoleId(), is("foo"));
    assertThat(role.getSource(), is(UserManager.DEFAULT_SOURCE));
    assertThat(role.getName(), is("bar"));
    assertThat(role.getDescription(), is("baz"));
    assertThat(role.getPrivileges(), contains("priv"));
    assertThat(role.getRoles(), contains("role"));
  }
}
