
package org.globalart.astrum.security.authz;

import org.apache.shiro.authz.permission.WildcardPermission;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

import static org.junit.Assert.assertTrue;

/**
 * Permission trials.
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class PermissionTrialTest
{
  @Test
  public void testImpliedWildcard() {
    WildcardPermission perm = new WildcardPermission("astrum:something:special:read");
    WildcardPermission granted = new WildcardPermission("astrum:*");
    assertTrue(granted.implies(perm));
  }

  @Test
  public void testImpliedWildcard_withoutAsterick() {
    WildcardPermission perm = new WildcardPermission("astrum:something:special:read");
    WildcardPermission granted = new WildcardPermission("astrum");
    assertTrue(granted.implies(perm));
  }
}
