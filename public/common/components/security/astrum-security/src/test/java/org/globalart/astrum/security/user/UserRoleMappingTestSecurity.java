
package org.globalart.astrum.security.user;

import org.globalart.astrum.security.config.MemorySecurityConfiguration;
import org.globalart.astrum.security.config.memory.MemoryCPrivilege.MemoryCPrivilegeBuilder;
import org.globalart.astrum.security.config.memory.MemoryCRole;
import org.globalart.astrum.security.config.memory.MemoryCUser;
import org.globalart.astrum.security.config.memory.MemoryCUserRoleMapping;

/**
 * @since 3.0
 */
public class UserRoleMappingTestSecurity
{
  public static MemorySecurityConfiguration securityModel() {
    return new MemorySecurityConfiguration()
        .withUsers(
            new MemoryCUser()
                .withId("admin")
                .withPassword("f865b53623b121fd34ee5426c792e5c33af8c227")
                .withFirstName("Administrator")
                .withStatus("active")
                .withEmail("admin@example.org"),
            new MemoryCUser()
                .withId("test-user")
                .withPassword("b2a0e378437817cebdf753d7dff3dd75483af9e0")
                .withFirstName("Test")
                .withLastName("User")
                .withStatus("active")
                .withEmail("test-user@example.org"),
            new MemoryCUser()
                .withId("test-user-with-empty-role")
                .withPassword("b2a0e378437817cebdf753d7dff3dd75483af9e0")
                .withFirstName("Test")
                .withLastName("User With Empty Role")
                .withStatus("active")
                .withEmail("test-user-with-empty-role@example.org"),
            new MemoryCUser()
                .withId("anonymous")
                .withPassword("0a92fab3230134cca6eadd9898325b9b2ae67998")
                .withFirstName("Anonynmous")
                .withLastName("User")
                .withStatus("active")
                .withEmail("anonymous@example.org"))
        .withUserRoleMappings(
            new MemoryCUserRoleMapping()
                .withUserId("other-user")
                .withSource("other-realm")
                .withRoles("role2", "role3"),
            new MemoryCUserRoleMapping()
                .withUserId("admin")
                .withSource("default")
                .withRoles("role1"),
            new MemoryCUserRoleMapping()
                .withUserId("test-user")
                .withSource("default")
                .withRoles("role1", "role2"),
            new MemoryCUserRoleMapping()
                .withUserId("test-user-with-empty-role")
                .withSource("default")
                .withRoles("empty-role", "role1", "role2"),
            new MemoryCUserRoleMapping()
                .withUserId("anonymous")
                .withSource("default")
                .withRoles("role2"))
        .withPrivileges(
            new MemoryCPrivilegeBuilder("1")
                .type("method")
                .name("1")
                .description("")
                .property("method", "read")
                .property("permission", "/some/path1/")
                .build(),
            new MemoryCPrivilegeBuilder("2")
                .type("method")
                .name("2")
                .description("")
                .property("method", "read")
                .property("permission", "/some/path2/")
                .build(),
            new MemoryCPrivilegeBuilder("3")
                .type("method")
                .name("3")
                .description("")
                .property("method", "read")
                .property("permission", "/some/path/")
                .build(),
            new MemoryCPrivilegeBuilder("4")
                .type("method")
                .name("4")
                .description("")
                .property("method", "read")
                .property("permission", "/some/path/")
                .build())
        .withRoles(
            new MemoryCRole()
                .withId("role1")
                .withName("RoleOne")
                .withDescription("Role One")
                .withPrivileges("1", "2"),
            new MemoryCRole()
                .withId("role2")
                .withName("RoleTwo")
                .withDescription("Role Two")
                .withPrivileges("3", "4"),
            new MemoryCRole()
                .withId("role3")
                .withName("RoleThree")
                .withDescription("Role Three")
                .withPrivileges("1", "4"),
            new MemoryCRole()
                .withId("empty-role")
                .withName("Empty Role")
                .withDescription("Empty Role"));
  }
}
