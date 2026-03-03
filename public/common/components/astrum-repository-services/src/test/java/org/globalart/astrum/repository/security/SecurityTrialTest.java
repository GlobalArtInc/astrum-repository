
package org.globalart.astrum.repository.security;

import org.apache.shiro.authz.permission.DomainPermission;
import org.apache.shiro.authz.permission.WildcardPermission;
import org.junit.Test;

import static org.junit.Assert.assertTrue;

public class SecurityTrialTest
{
  @Test
  public void wildcardPermissionString() {
    assertDoesNotThrowException(() -> new WildcardPermission("foo:bar:*:baz"));
  }

  @Test
  public void domainPermissionString() {
    assertDoesNotThrowException(() -> new DomainPermission("foo,bar", "read,write"));
  }

  private static class CustomPermission
      extends DomainPermission
  {
    CustomPermission(final String actions, final String targets) {
      super(actions, targets);
    }
  }

  @Test
  public void customDomainPermissionString() {
    assertDoesNotThrowException(() -> new CustomPermission("foo,bar", "read,write"));
  }

  @Test
  public void impliedPermission() {
    WildcardPermission granted = new WildcardPermission("test:*");
    WildcardPermission permission = new WildcardPermission("test:foo");
    assertTrue(granted.implies(permission));
  }

  private void assertDoesNotThrowException(Runnable r) {
    try {
      r.run();
    }
    catch (Exception e) {
      throw new AssertionError("Expected no exception, but got: " + e, e);
    }
  }
}
