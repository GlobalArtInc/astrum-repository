
package org.globalart.astrum.plugins.defaultrole.internal;

import org.globalart.astrum.plugins.defaultrole.DefaultRoleRealm;
import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.realm.RealmManager;
import org.globalart.astrum.security.role.Role;

import com.codahale.metrics.health.HealthCheck.Result;
import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static java.util.Collections.singleton;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.security.user.UserManager.DEFAULT_SOURCE;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class DefaultRoleHealthCheckTest
{
  @Mock
  private RealmManager realmManager;

  @Mock
  private DefaultRoleRealm defaultRoleRealm;

  @Mock
  private SecuritySystem securitySystem;

  private DefaultRoleHealthCheck underTest;

  @Before
  public void setup() {
    underTest = new DefaultRoleHealthCheck(realmManager, defaultRoleRealm, securitySystem);
  }

  @Test
  public void testCheck_notConfiguredNotEnabled() throws Exception {
    Result result = underTest.check();
    assertThat(result.isHealthy(), is(true));
  }

  @Test
  public void testCheck_notConfiguredIsEnabled() throws Exception {
    when(realmManager.isRealmEnabled(DefaultRoleRealm.NAME)).thenReturn(true);

    Result result = underTest.check();
    assertThat(result.isHealthy(), is(false));
  }

  @Test
  public void testCheck_isConfiguredIsEnabledRoleMissing() throws Exception {
    when(realmManager.isRealmEnabled(DefaultRoleRealm.NAME)).thenReturn(true);
    when(defaultRoleRealm.getRole()).thenReturn("test-role");

    Result result = underTest.check();
    assertThat(result.isHealthy(), is(false));
  }

  @Test
  public void testCheck_isConfiguredIsEnabledRoleAvailable() throws Exception {
    when(realmManager.isRealmEnabled(DefaultRoleRealm.NAME)).thenReturn(true);
    when(defaultRoleRealm.getRole()).thenReturn("test-role");

    Role role = mock(Role.class);
    when(role.getRoleId()).thenReturn("test-role");
    when(securitySystem.listRoles(DEFAULT_SOURCE)).thenReturn(singleton(role));

    Result result = underTest.check();
    assertThat(result.isHealthy(), is(true));
  }
}
