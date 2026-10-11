package org.globalart.astrum.security.accesstoken;

import org.apache.shiro.authz.permission.AllPermission;
import org.apache.shiro.authz.permission.WildcardPermission;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.junit.Assert.assertThrows;

public class AccessTokenScopeTest
{
  @Test
  public void readScopeAllowsOnlyReadingAndBrowsing() {
    assertThat(read("astrum:repository-view:docker:images:read"), is(true));
    assertThat(read("astrum:repository-view:docker:images:browse"), is(true));
    assertThat(read("astrum:repository-view:docker:images:read,browse"), is(true));
    assertThat(read("astrum:search:read"), is(true));
    assertThat(read("astrum:repository-view:docker:images:add"), is(false));
    assertThat(read("astrum:repository-view:docker:images:edit"), is(false));
    assertThat(read("astrum:repository-view:docker:images:read,delete"), is(false));
    assertThat(read("astrum:repository-view:*:*:*"), is(false));
    assertThat(read("astrum:*"), is(false));
    assertThat(AccessTokenScope.READ.permits(new AllPermission()), is(false));
  }

  @Test
  public void writeScopeAllowsEverything() {
    assertThat(AccessTokenScope.WRITE.permits(new WildcardPermission("astrum:repository-view:raw:files:add")),
        is(true));
    assertThat(AccessTokenScope.WRITE.permits(new AllPermission()), is(true));
  }

  @Test
  public void scopesAreIdentifiedByLowerCaseNames() {
    assertThat(AccessTokenScope.fromId("read"), is(AccessTokenScope.READ));
    assertThat(AccessTokenScope.fromId("WRITE"), is(AccessTokenScope.WRITE));
    assertThat(AccessTokenScope.WRITE.id(), is("write"));
    assertThrows(IllegalArgumentException.class, () -> AccessTokenScope.fromId("admin"));
  }

  @Test
  public void principalWithUnknownScopePermitsNothing() {
    AccessTokenPrincipal principal = new AccessTokenPrincipal("id", "ci", "admin");
    assertThat(principal.permits(new WildcardPermission("astrum:search:read")), is(false));
  }

  private static boolean read(final String permission) {
    return AccessTokenScope.READ.permits(new WildcardPermission(permission));
  }
}
