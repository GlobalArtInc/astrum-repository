
package org.globalart.astrum.coreui.internal.ldap;

import java.util.Map;

import org.globalart.astrum.security.realm.RealmManager;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasEntry;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class LdapStateContributorTest
{
  private static final String LDAP_REALM_NAME = "LdapRealm";

  @Mock
  private RealmManager realmManager;

  private LdapStateContributor underTest;

  @Before
  public void setUp() {
    underTest = new LdapStateContributor(false, 3, realmManager);
  }

  @Test
  public void shouldReturnLdapRealmDisabledWhenRealmIsNotEnabled() {
    when(realmManager.isRealmEnabled(LDAP_REALM_NAME)).thenReturn(false);

    Map<String, Object> state = underTest.getState();

    assertThat(state, hasEntry(is("ldapRealmEnabled"), is(false)));
    verify(realmManager).isRealmEnabled(LDAP_REALM_NAME);
  }

  @Test
  public void shouldReturnLdapRealmEnabledWhenRealmIsEnabled() {
    when(realmManager.isRealmEnabled(LDAP_REALM_NAME)).thenReturn(true);

    Map<String, Object> state = underTest.getState();

    assertThat(state, hasEntry(is("ldapRealmEnabled"), is(true)));
    verify(realmManager).isRealmEnabled(LDAP_REALM_NAME);
  }

  @Test
  public void shouldReturnFeatureFlagsInState() {
    when(realmManager.isRealmEnabled(LDAP_REALM_NAME)).thenReturn(false);

    Map<String, Object> state = underTest.getState();

    assertThat(state, hasEntry(is("astrum.react.ldap"), is(false)));
    assertThat(state, hasEntry(is("astrum.ldap.mapped.role.query.character.limit"), is(3)));
  }
}
