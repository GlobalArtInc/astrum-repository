
package org.globalart.astrum.security.aspect;

import org.globalart.astrum.testcommon.extensions.AuthenticationExtension;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension.WithUser;

import org.apache.shiro.authz.AuthorizationException;
import org.apache.shiro.authz.UnauthenticatedException;
import org.apache.shiro.authz.UnauthorizedException;
import org.apache.shiro.authz.annotation.Logical;
import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertThrows;

@ExtendWith(AuthenticationExtension.class)
class SecurityAspectsTest
{
  @Test
  @WithUser
  void testMethodRequiresAuthentication() {
    assertDoesNotThrow(
        this::methodRequiringAuthentication,
        "Method should not throw an exception when the subject is authenticated");
  }

  @Test
  @WithUser(isAuthenticated = false)
  void testMethodThrowsExceptionWhenAUserIsNotAuthenticated() {
    assertThrows(UnauthenticatedException.class, this::methodRequiringAuthentication);
  }

  @Test
  @WithUser(permissions = "astrum:other:*")
  void testExceptionRequiresPermissions() {
    assertThrows(AuthorizationException.class,
        this::methodRequiresPermission,
        "Method should throw an exception when the user doesn't have the required permissions");
  }

  @Test
  @WithUser(permissions = "astrum:some:*")
  void testRequiresPermissions() {
    assertDoesNotThrow(
        this::methodRequiresPermission,
        "Method should not throw an exception when the user has the required permissions");
  }

  @Test
  @WithUser(permissions = {"astrum:capabilities:view", "astrum:capabilities:read"})
  void testRequiresMultiplePermissionsOnMethod() {
    assertThrows(UnauthorizedException.class,
        this::methodRequiresMultiplePermissions,
        "Method should throw an exception when the user doesn't have all the required permissions");
  }

  @Test
  @WithUser(permissions = {"astrum:capabilities:view"})
  void testMethodAllowsUserWithAtLeastOnePermission() {
    assertDoesNotThrow(
        this::methodAllowsUserWithAtLeastOnePermission,
        "Method should throw an exception when the user doesn't have all the required permissions");
  }

  @RequiresAuthentication
  public String methodRequiringAuthentication() {
    return "authenticated";
  }

  @RequiresPermissions("astrum:some:permission")
  public String methodRequiresPermission() {
    return "requires some permissions";
  }

  @RequiresPermissions(value = {"astrum:capabilities:write", "astrum:capabilities:read"}, logical = Logical.AND)
  public String methodRequiresMultiplePermissions() {
    return "requires multiple permissions";
  }

  @RequiresPermissions(value = {"astrum:capabilities:view", "astrum:capabilities:read"}, logical = Logical.OR)
  public String methodAllowsUserWithAtLeastOnePermission() {
    return "requires multiple permissions";
  }
}
