
package org.globalart.astrum.coreui;

import org.globalart.astrum.security.anonymous.AnonymousConfiguration;
import org.globalart.astrum.security.anonymous.AnonymousManager;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension.WithUser;
import org.globalart.astrum.testcommon.validation.ValidationExtension;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.InjectMocks;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
@ExtendWith(ValidationExtension.class)
@ExtendWith(AuthenticationExtension.class)
@WithUser
class AnonymousSettingsResourceTest
{
  @Mock
  private AnonymousManager anonymousManager;

  @Mock
  private AnonymousConfiguration anonymousConfiguration;

  @InjectMocks
  private AnonymousSettingsResource underTest;

  @Test
  void readReturnsCurrentConfiguration() {
    when(anonymousManager.getConfiguration()).thenReturn(anonymousConfiguration);
    when(anonymousConfiguration.isEnabled()).thenReturn(true);
    when(anonymousConfiguration.getUserId()).thenReturn("anonymous");
    when(anonymousConfiguration.getRealmName()).thenReturn("AstrumAuthorizingRealm");

    AnonymousSettingsXO result = underTest.read();

    assertThat(result, is(notNullValue()));
    assertThat(result.getEnabled(), is(true));
    assertThat(result.getUserId(), is("anonymous"));
    assertThat(result.getRealmName(), is("AstrumAuthorizingRealm"));
  }

  @Test
  void readReturnsDisabledConfiguration() {
    when(anonymousManager.getConfiguration()).thenReturn(anonymousConfiguration);
    when(anonymousConfiguration.isEnabled()).thenReturn(false);
    when(anonymousConfiguration.getUserId()).thenReturn("anon-user");
    when(anonymousConfiguration.getRealmName()).thenReturn("TestRealm");

    AnonymousSettingsXO result = underTest.read();

    assertThat(result, is(notNullValue()));
    assertThat(result.getEnabled(), is(false));
    assertThat(result.getUserId(), is("anon-user"));
    assertThat(result.getRealmName(), is("TestRealm"));
  }

  @Test
  void updateSetsConfigurationOnManager() {
    AnonymousConfiguration newConfig = setupMockNewConfiguration();

    AnonymousSettingsXO xo = new AnonymousSettingsXO();
    xo.setEnabled(true);
    xo.setUserId("anonymous");
    xo.setRealmName("AstrumAuthorizingRealm");

    underTest.update(xo);

    verify(anonymousManager).newConfiguration();
    verify(newConfig).setEnabled(true);
    verify(newConfig).setUserId("anonymous");
    verify(newConfig).setRealmName("AstrumAuthorizingRealm");
    verify(anonymousManager).setConfiguration(newConfig);
  }

  @Test
  void updateWithDisabledConfiguration() {
    AnonymousConfiguration newConfig = setupMockNewConfiguration();

    AnonymousSettingsXO xo = new AnonymousSettingsXO();
    xo.setEnabled(false);
    xo.setUserId("test-user");
    xo.setRealmName("TestRealm");

    underTest.update(xo);

    verify(newConfig).setEnabled(false);
    verify(newConfig).setUserId("test-user");
    verify(newConfig).setRealmName("TestRealm");
    verify(anonymousManager).setConfiguration(newConfig);
  }

  private AnonymousConfiguration setupMockNewConfiguration() {
    AnonymousConfiguration newConfig = org.mockito.Mockito.mock(AnonymousConfiguration.class);
    when(anonymousManager.newConfiguration()).thenReturn(newConfig);
    return newConfig;
  }
}
