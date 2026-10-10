package org.globalart.astrum.ldap.internal;

import java.util.Optional;
import java.util.Set;

import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.security.authz.PrincipalPermissionsCacheFactory;
import org.globalart.astrum.security.role.RoleIdentifier;
import org.globalart.astrum.security.user.User;
import org.globalart.astrum.security.user.UserNotFoundException;
import org.globalart.astrum.security.user.UserNotFoundTransientException;

import org.apache.shiro.authc.AuthenticationInfo;
import org.apache.shiro.authc.UnknownAccountException;
import org.apache.shiro.authc.UsernamePasswordToken;
import org.apache.shiro.authz.AuthorizationInfo;
import org.apache.shiro.subject.SimplePrincipalCollection;
import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.junit.Assert.assertThrows;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

public class LdapRealmTest
{
  private static final LdapUser ALICE =
      new LdapUser("alice", "uid=alice,ou=people,dc=example,dc=com", "Alice Smith", "alice@example.com",
          Set.of("developers", "admins"));

  private LdapManager ldapManager;

  private LdapRealm realm;

  private LdapUserManager userManager;

  @Before
  public void setUp() {
    ldapManager = mock(LdapManager.class);
    when(ldapManager.isConfigured()).thenReturn(true);
    realm = new LdapRealm(mock(PrincipalPermissionsCacheFactory.class), ldapManager);
    userManager = new LdapUserManager(mock(EventManager.class), ldapManager);
  }

  @Test
  public void authenticationUsesTheNameStoredInLdap() {
    when(ldapManager.authenticate(eq("ALICE"), any())).thenReturn(ALICE);
    AuthenticationInfo info = realm.getAuthenticationInfo(new UsernamePasswordToken("ALICE", "pw"));
    assertThat(info.getPrincipals().getPrimaryPrincipal(), is("alice"));
    assertThat(info.getPrincipals().getRealmNames(), containsInAnyOrder(LdapRealm.NAME));
  }

  @Test
  public void notConfiguredMeansNotOurs() {
    when(ldapManager.isConfigured()).thenReturn(false);
    assertThrows(UnknownAccountException.class,
        () -> realm.getAuthenticationInfo(new UsernamePasswordToken("alice", "pw")));
  }

  @Test
  public void groupsBecomeRoles() {
    when(ldapManager.getUser("alice")).thenReturn(Optional.of(ALICE));
    AuthorizationInfo info = realm.doGetAuthorizationInfo(new SimplePrincipalCollection("alice", LdapRealm.NAME));
    assertThat(info.getRoles(), containsInAnyOrder("developers", "admins"));
    assertThat(realm.doGetAuthorizationInfo(new SimplePrincipalCollection("alice", "OtherRealm")), nullValue());
  }

  @Test
  public void userManagerMapsUsers() throws Exception {
    when(ldapManager.getUser("alice")).thenReturn(Optional.of(ALICE));
    User user = userManager.getUser("alice");
    assertThat(user.getSource(), is("LDAP"));
    assertThat(user.getFirstName(), is("Alice"));
    assertThat(user.getLastName(), is("Smith"));
    assertThat(user.isReadOnly(), is(true));
    assertThat(user.getRoles(), containsInAnyOrder(new RoleIdentifier("LDAP", "developers"),
        new RoleIdentifier("LDAP", "admins")));
  }

  @Test
  public void unreachableLdapIsNotAMissingUser() {
    when(ldapManager.getUser("alice")).thenThrow(new LdapServerException("down"));
    when(ldapManager.getUser("nobody")).thenReturn(Optional.empty());
    assertThrows(UserNotFoundTransientException.class, () -> userManager.getUser("alice"));
    UserNotFoundException missing = assertThrows(UserNotFoundException.class, () -> userManager.getUser("nobody"));
    assertThat(missing instanceof UserNotFoundTransientException, is(false));
  }
}
