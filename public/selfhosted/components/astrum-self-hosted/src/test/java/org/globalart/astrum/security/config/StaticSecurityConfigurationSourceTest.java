
package org.globalart.astrum.security.config;

import org.apache.shiro.authc.credential.PasswordService;
import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class StaticSecurityConfigurationSourceTest
{
  @Mock
  private PasswordService passwordService;

  @Mock
  private AdminPasswordSource adminPasswordSource;

  private StaticSecurityConfigurationSource underTest;

  @Before
  public void setup() throws Exception {
    underTest = new StaticSecurityConfigurationSource(passwordService, adminPasswordSource, true);
    when(passwordService.encryptPassword(any())).thenReturn("encrypted");
  }

  @Test
  public void shouldGetPasswordFromEnvironmentVariable() {
    String password = "supersecretpassword";

    underTest = new StaticSecurityConfigurationSource(passwordService, adminPasswordSource, false, password);

    SecurityConfiguration configuration = underTest.getConfiguration();
    CUser user = configuration.getUser("admin");
    assertThat(user.getPassword(), is("encrypted"));
    verify(passwordService).encryptPassword(password);
  }

  @Test
  public void shouldGeneratePassword() {
    SecurityConfiguration configuration = underTest.getConfiguration();

    CUser user = configuration.getUser("admin");

    assertThat(user.getPassword(), is("encrypted"));
    verify(adminPasswordSource).getPassword(true);
    verify(passwordService, times(1)).encryptPassword(any());
  }

  @Test
  public void testGetConfiguration_adminUserStatusCheck() {
    SecurityConfiguration configuration = underTest.getConfiguration();
    CUser user = configuration.getUser("admin");
    assertThat(user.getStatus(), is(CUser.STATUS_CHANGE_PASSWORD));
  }

  @Test
  public void testGetConfiguration_adminUserStatusCheckNonRandom() {
    underTest = new StaticSecurityConfigurationSource(passwordService, adminPasswordSource, false);
    SecurityConfiguration configuration = underTest.getConfiguration();
    CUser user = configuration.getUser("admin");
    assertThat(user.getStatus(), is(CUser.STATUS_ACTIVE));
  }
}
