
package org.globalart.astrum.security.role;

import java.util.Arrays;
import java.util.Collections;

import org.globalart.astrum.security.Roles;
import org.globalart.astrum.security.SecurityHelper;

import org.apache.shiro.authz.Permission;
import org.apache.shiro.authz.permission.RolePermissionResolver;
import org.apache.shiro.authz.permission.WildcardPermission;
import org.apache.shiro.subject.Subject;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class RoleAssignabilityCheckerTest
{
  @Mock
  private RolePermissionResolver rolePermissionResolver;

  @Mock
  private SecurityHelper securityHelper;

  @Mock
  private Subject subject;

  private RoleAssignabilityChecker underTest;

  @BeforeEach
  void setup() {
    when(securityHelper.subject()).thenReturn(subject);
    underTest = new RoleAssignabilityChecker(rolePermissionResolver, securityHelper);
  }

  @Test
  void adminRole_assignableToAdminSubject() {
    when(subject.hasRole(Roles.ADMIN_ROLE_ID)).thenReturn(true);

    assertThat(underTest.isRoleAssignable(Roles.ADMIN_ROLE_ID), is(true));
  }

  @Test
  void adminRole_notAssignableToNonAdminSubject() {
    when(subject.hasRole(Roles.ADMIN_ROLE_ID)).thenReturn(false);

    assertThat(underTest.isRoleAssignable(Roles.ADMIN_ROLE_ID), is(false));
  }

  @Test
  void nonAdminRole_assignableWhenSubjectHoldsAllPermissions() {
    Permission perm = new WildcardPermission("astrum:users:read");
    when(rolePermissionResolver.resolvePermissionsInRole("as-viewer")).thenReturn(Collections.singletonList(perm));
    when(subject.isPermitted(perm)).thenReturn(true);

    assertThat(underTest.isRoleAssignable("as-viewer"), is(true));
  }

  @Test
  void nonAdminRole_notAssignableWhenSubjectLacksAPermission() {
    Permission allowed = new WildcardPermission("astrum:users:read");
    Permission denied = new WildcardPermission("astrum:users:create");
    when(rolePermissionResolver.resolvePermissionsInRole("as-editor"))
        .thenReturn(Arrays.asList(allowed, denied));
    when(subject.isPermitted(allowed)).thenReturn(true);
    when(subject.isPermitted(denied)).thenReturn(false);

    assertThat(underTest.isRoleAssignable("as-editor"), is(false));
  }

  @Test
  void roleWithNoPermissions_alwaysAssignable() {
    when(rolePermissionResolver.resolvePermissionsInRole("empty-role")).thenReturn(Collections.emptyList());

    assertThat(underTest.isRoleAssignable("empty-role"), is(true));
  }
}
