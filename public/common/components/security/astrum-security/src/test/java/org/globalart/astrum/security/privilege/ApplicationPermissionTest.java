
package org.globalart.astrum.security.privilege;

import org.apache.shiro.authz.Permission;
import org.apache.shiro.authz.permission.PermissionResolver;
import org.apache.shiro.authz.permission.WildcardPermissionResolver;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

/**
 * Tests for {@link ApplicationPermission}.
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class ApplicationPermissionTest
{
  private final PermissionResolver permissionResolver = new WildcardPermissionResolver();

  @Test
  public void simpleApplicationPermissionsMatch() {

    Permission authorizedPermission = new ApplicationPermission("feature", "action", "anotherAction");

    // simulate @RequiresPermissions resolution on UI resources
    Permission requiredPermission = permissionResolver.resolvePermission("astrum:feature:action");

    assertThat(authorizedPermission.implies(requiredPermission), is(true));
  }

  @Test
  public void complexApplicationPermissionsMatch() {

    Permission authorizedPermission = new ApplicationPermission("feature:method", "action", "anotherAction");

    // simulate @RequiresPermissions resolution on UI resources
    Permission requiredPermission = permissionResolver.resolvePermission("astrum:feature:method:action");

    assertThat(authorizedPermission.implies(requiredPermission), is(true));
  }
}
