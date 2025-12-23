
package org.globalart.astrum.security;

import org.globalart.astrum.security.config.MemorySecurityConfiguration;
import org.globalart.astrum.security.config.memory.MemoryCUser;
import org.globalart.astrum.security.config.memory.MemoryCUserRoleMapping;

/**
 * @since 3.0
 */
public class BaseSecurityConfig
{
  public static MemorySecurityConfiguration get() {
    return new MemorySecurityConfiguration().withUsers(
        new MemoryCUser().withId("admin")
            .withPassword("f865b53623b121fd34ee5426c792e5c33af8c227")
            .withFirstName("Administrator")
            .withStatus("active")
            .withEmail("admin@example.org"),
        new MemoryCUser().withId("deployment")
            .withPassword("b2a0e378437817cebdf753d7dff3dd75483af9e0")
            .withFirstName("Deployment")
            .withLastName("User")
            .withStatus("active")
            .withEmail("deployment@example.org"),
        new MemoryCUser().withId("anonymous")
            .withPassword("0a92fab3230134cca6eadd9898325b9b2ae67998")
            .withFirstName("Astrum")
            .withLastName("Anonynmous User")
            .withStatus("active")
            .withEmail("anonymous@example.org"))
        .withUserRoleMappings(new MemoryCUserRoleMapping().withUserId("admin").withSource("default").withRoles("admin"),
            new MemoryCUserRoleMapping().withUserId("deployment")
                .withSource("default")
                .withRoles("deployment",
                    "repo-all-full"),
            new MemoryCUserRoleMapping().withUserId("anonymous")
                .withSource("default")
                .withRoles("anonymous",
                    "repo-all-read"));
  }
}
