
package org.globalart.astrum.security.config;

import org.globalart.astrum.security.internal.SecurityConfigurationCleanerImpl;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

import static org.junit.Assert.assertFalse;

/**
 * Tests for {@link SecurityConfigurationCleanerImpl}.
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class SecuritySettingsCleanerImplTest
{
  private SecurityConfigurationCleanerImpl underTest;

  private MemorySecurityConfiguration configuration;

  @Before
  public void setUp() throws Exception {
    underTest = new SecurityConfigurationCleanerImpl();
    configuration = DefaultSecurityConfigurationCleanerTestSecurity.securityModel();
  }

  @Test
  public void testRemovePrivilege() throws Exception {
    String privilegeId = configuration.getPrivileges().get(0).getId();
    configuration.removePrivilege(privilegeId);

    underTest.privilegeRemoved(configuration, privilegeId);

    for (CRole role : configuration.getRoles()) {
      assertFalse(role.getPrivileges().contains(privilegeId));
    }
  }

  @Test
  public void testRemoveRole() throws Exception {
    String roleId = configuration.getRoles().get(0).getId();
    configuration.removeRole(roleId);

    underTest.roleRemoved(configuration, roleId);

    for (CRole crole : configuration.getRoles()) {
      assertFalse(crole.getPrivileges().contains(roleId));
    }

    for (CUserRoleMapping mapping : configuration.getUserRoleMappings()) {
      assertFalse(mapping.getRoles().contains(roleId));
    }
  }
}
