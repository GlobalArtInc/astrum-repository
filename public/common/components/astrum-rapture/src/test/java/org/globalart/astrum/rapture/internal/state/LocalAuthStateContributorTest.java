
package org.globalart.astrum.rapture.internal.state;

import java.util.Map;

import org.globalart.astrum.security.realm.RealmManager;

import org.junit.Test;
import org.mockito.InjectMocks;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasEntry;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.security.internal.DefaultRealmConstants.DEFAULT_REALM_NAME;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class LocalAuthStateContributorTest
{
  @Mock
  private RealmManager realmManager;

  @InjectMocks
  private LocalAuthStateContributor underTest;

  @Test
  public void shouldReturnLocalAuthRealmDisabledWhenRealmIsNotEnabled() {
    when(realmManager.isRealmEnabled(DEFAULT_REALM_NAME)).thenReturn(false);

    Map<String, Object> state = underTest.getState();

    assertThat(state, hasEntry(is("localAuthRealmEnabled"), is(false)));
    verify(realmManager).isRealmEnabled(DEFAULT_REALM_NAME);
  }

  @Test
  public void shouldReturnLocalAuthRealmEnabledWhenRealmIsEnabled() {
    when(realmManager.isRealmEnabled(DEFAULT_REALM_NAME)).thenReturn(true);

    Map<String, Object> state = underTest.getState();

    assertThat(state, hasEntry(is("localAuthRealmEnabled"), is(true)));
    verify(realmManager).isRealmEnabled(DEFAULT_REALM_NAME);
  }
}
