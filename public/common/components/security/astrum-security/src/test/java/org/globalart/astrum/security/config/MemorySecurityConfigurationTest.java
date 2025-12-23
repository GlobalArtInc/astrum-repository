
package org.globalart.astrum.security.config;

import org.globalart.astrum.security.config.memory.MemoryCUserRoleMapping;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.Parameterized;
import org.junit.runners.Parameterized.Parameter;
import org.junit.runners.Parameterized.Parameters;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

@RunWith(Parameterized.class)
public class MemorySecurityConfigurationTest
{
  @Parameters(name = "userRoleMappings for source: '{0}' read isFound: {1}")
  public static Object[][] params() {
    return new Object[][]{{"default", false}, {"ldap", true}, {"crowd", true}, {"other", false}};
  }

  @Parameter
  public String src;

  @Parameter(1)
  public boolean isFound;

  private MemorySecurityConfiguration config;

  @Before
  public void setup() {
    config = new MemorySecurityConfiguration();
  }

  @Test
  public void testGetUserRoleMapping() {
    MemoryCUserRoleMapping newUserRoleMapping =
        new MemoryCUserRoleMapping().withUserId("userid").withSource(src).withRoles("test-role");
    config.addUserRoleMapping(newUserRoleMapping);

    CUserRoleMapping roleMapping = config.getUserRoleMapping("USERID", src);

    assertThat(roleMapping != null, is(isFound));

    if (isFound) {
      roleMapping.setRoles(newUserRoleMapping.getRoles());
    }
  }
}
