
package org.globalart.astrum.repository.security.internal;

import org.globalart.astrum.security.config.CUser;
import org.globalart.astrum.security.config.SecurityConfigurationManager;
import org.globalart.astrum.security.internal.AuthenticatingRealmImpl;
import org.globalart.astrum.security.realm.RealmManager;
import org.globalart.astrum.security.user.UserNotFoundException;

import com.codahale.metrics.health.HealthCheck.Result;
import org.apache.shiro.authc.credential.PasswordService;
import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class DefaultUserHealthCheckTest
{
  @Mock
  private RealmManager realmManager;

  @Mock
  private SecurityConfigurationManager securityConfigurationManager;

  @Mock
  private PasswordService passwordService;

  @Mock
  private CUser adminUser;

  private DefaultUserHealthCheck underTest;

  @Before
  public void setUp() throws UserNotFoundException {
    underTest = new DefaultUserHealthCheck(realmManager, securityConfigurationManager, passwordService);
    when(realmManager.isRealmEnabled(AuthenticatingRealmImpl.NAME)).thenReturn(true);
    when(securityConfigurationManager.readUser("admin")).thenReturn(adminUser);
    when(adminUser.getPassword()).thenReturn("some-password");
  }

  @Test
  public void checkIsHealthyWhenRealmIsDisabled() {
    when(realmManager.isRealmEnabled(AuthenticatingRealmImpl.NAME)).thenReturn(false);
    assertThat(underTest.check().isHealthy()).isTrue();
  }

  @Test
  public void checkIsHealthyWhenPasswordChanged() {
    when(passwordService.passwordsMatch("admin123", "some-password")).thenReturn(false);
    assertThat(underTest.check().isHealthy()).isTrue();
  }

  @Test
  public void checkIsHealthyWhenUserNotFound() throws UserNotFoundException {
    when(securityConfigurationManager.readUser("admin")).thenThrow(new UserNotFoundException("admin"));
    assertThat(underTest.check().isHealthy()).isTrue();
  }

  @Test
  public void checkIsUnhealthy() {
    when(passwordService.passwordsMatch("admin123", "some-password")).thenReturn(true);
    Result result = underTest.check();
    assertThat(result.isHealthy()).isFalse();
    assertThat(result.getMessage()).isEqualTo(DefaultUserHealthCheck.ERROR_MESSAGE);
  }
}
