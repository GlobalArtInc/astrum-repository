
package org.globalart.astrum.coreui;

import java.util.List;

import org.globalart.astrum.security.realm.RealmManager;
import org.globalart.astrum.security.realm.SecurityRealm;
import org.globalart.astrum.security.user.UserManager;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension.WithUser;
import org.globalart.astrum.testcommon.validation.ValidationExtension;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.empty;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
@ExtendWith(ValidationExtension.class)
@ExtendWith(AuthenticationExtension.class)
@WithUser
class RealmSettingsResourceTest
{
  @Mock
  private RealmManager realmManager;

  @Test
  void readRealmTypesFiltersToAuthenticationRealms() {
    UserManager userManager1 = mock(UserManager.class);
    when(userManager1.getAuthenticationRealmName()).thenReturn("realm-A");
    UserManager userManager2 = mock(UserManager.class);
    when(userManager2.getAuthenticationRealmName()).thenReturn("realm-B");

    SecurityRealm realmA = new SecurityRealm("realm-A", "Realm A");
    SecurityRealm realmB = new SecurityRealm("realm-B", "Realm B");
    SecurityRealm realmC = new SecurityRealm("realm-C", "Realm C");

    when(realmManager.getAvailableRealms(true)).thenReturn(List.of(realmA, realmB, realmC));

    RealmSettingsResource underTest = new RealmSettingsResource(realmManager, List.of(userManager1, userManager2));
    List<SecurityRealm> result = underTest.readRealmTypes();

    assertThat(result, hasSize(2));
    assertThat(result.get(0).getId(), is("realm-A"));
    assertThat(result.get(1).getId(), is("realm-B"));
  }

  @Test
  void readRealmTypesReturnsEmptyWhenNoRealmsMatch() {
    UserManager userManager = mock(UserManager.class);
    when(userManager.getAuthenticationRealmName()).thenReturn("realm-X");

    SecurityRealm realmA = new SecurityRealm("realm-A", "Realm A");
    SecurityRealm realmB = new SecurityRealm("realm-B", "Realm B");

    when(realmManager.getAvailableRealms(true)).thenReturn(List.of(realmA, realmB));

    RealmSettingsResource underTest = new RealmSettingsResource(realmManager, List.of(userManager));
    List<SecurityRealm> result = underTest.readRealmTypes();

    assertThat(result, is(empty()));
  }

  @Test
  void readRealmTypesReturnsEmptyWhenNoAvailableRealms() {
    UserManager userManager = mock(UserManager.class);
    when(userManager.getAuthenticationRealmName()).thenReturn("realm-A");

    when(realmManager.getAvailableRealms(true)).thenReturn(List.of());

    RealmSettingsResource underTest = new RealmSettingsResource(realmManager, List.of(userManager));
    List<SecurityRealm> result = underTest.readRealmTypes();

    assertThat(result, is(empty()));
  }

  @Test
  void readRealmTypesWithNoUserManagers() {
    SecurityRealm realmA = new SecurityRealm("realm-A", "Realm A");

    when(realmManager.getAvailableRealms(true)).thenReturn(List.of(realmA));

    RealmSettingsResource underTest = new RealmSettingsResource(realmManager, List.of());
    List<SecurityRealm> result = underTest.readRealmTypes();

    assertThat(result, is(empty()));
  }

  @Test
  void readRealmTypesSkipsUserManagersWithNullRealmName() {
    UserManager userManager1 = mock(UserManager.class);
    when(userManager1.getAuthenticationRealmName()).thenReturn(null);
    UserManager userManager2 = mock(UserManager.class);
    when(userManager2.getAuthenticationRealmName()).thenReturn("realm-B");

    SecurityRealm realmA = new SecurityRealm("realm-A", "Realm A");
    SecurityRealm realmB = new SecurityRealm("realm-B", "Realm B");

    when(realmManager.getAvailableRealms(true)).thenReturn(List.of(realmA, realmB));

    RealmSettingsResource underTest = new RealmSettingsResource(realmManager, List.of(userManager1, userManager2));
    List<SecurityRealm> result = underTest.readRealmTypes();

    assertThat(result, hasSize(1));
    assertThat(result.get(0).getId(), is("realm-B"));
  }
}
