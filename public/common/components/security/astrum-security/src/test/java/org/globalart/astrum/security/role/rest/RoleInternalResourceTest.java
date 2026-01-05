
package org.globalart.astrum.security.role.rest;

import java.util.Arrays;
import java.util.HashSet;
import java.util.List;
import jakarta.ws.rs.core.Response.Status;

import org.globalart.astrum.rest.WebApplicationMessageException;
import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.authz.NoSuchAuthorizationManagerException;
import org.globalart.astrum.security.role.Role;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension.WithUser;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertThrows;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
@ExtendWith(AuthenticationExtension.class)
@WithUser
class RoleInternalResourceTest
{
  @Mock
  private SecuritySystem securitySystem;

  private RoleInternalResource underTest;

  @BeforeEach
  void setup() {
    underTest = new RoleInternalResource(securitySystem);
  }

  @Test
  void testListRolesOnEmptySource() {
    Role r1 = createRole("role1", Arrays.asList("roleA", "roleB"), Arrays.asList("read", "write"));
    Role r2 = createRole("role2", Arrays.asList("roleC", "roleD"), Arrays.asList("read", "delete"));
    when(securitySystem.listRoles()).thenReturn(new HashSet<>(Arrays.asList(r1, r2)));

    List<RoleXOResponse> res = underTest.searchRoles("", "searchParam");

    verify(securitySystem).listRoles();
    assertEquals(2, res.size());
    assertEquals("role1", res.get(0).getName());
    assertEquals("role2", res.get(1).getName());
  }

  @Test
  void testSearchRoles() throws NoSuchAuthorizationManagerException {
    Role r1 = createRole("customRole1", Arrays.asList("roleA", "roleB"), Arrays.asList("read", "write"));
    Role r2 = createRole("customRole2", Arrays.asList("roleC", "roleD"), Arrays.asList("read", "delete"));
    Role r3 = createRole("customRole3", Arrays.asList("roleX", "roleY"), Arrays.asList("read", "delete"));
    when(securitySystem.searchRoles(anyString(), anyString())).thenReturn(new HashSet<>(Arrays.asList(r1, r2, r3)));

    List<RoleXOResponse> res = underTest.searchRoles("customSource", "searchParam");

    verify(securitySystem, never()).listRoles();
    verify(securitySystem).searchRoles("customSource", "searchParam");
    assertEquals(3, res.size());
    assertEquals("customRole1", res.get(0).getName());
    assertEquals("customRole2", res.get(1).getName());
    assertEquals("customRole3", res.get(2).getName());
  }

  @Test
  void testExceptionThrown() throws NoSuchAuthorizationManagerException {
    when(securitySystem.searchRoles(anyString(), anyString())).thenThrow(
        new NoSuchAuthorizationManagerException("bad source"));
    WebApplicationMessageException ex =
        assertThrows(WebApplicationMessageException.class, () -> underTest.searchRoles("authSource", "searchParam"));
    assertEquals(Status.NOT_FOUND.getStatusCode(), ex.getResponse().getStatus());
  }

  private Role createRole(String roleName, List<String> roles, List<String> privileges) {
    return new Role(roleName + "Id", roleName, roleName, "internal", true,
        new HashSet<>(roles), new HashSet<>(privileges));
  }
}
