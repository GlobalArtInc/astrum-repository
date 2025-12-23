
package org.globalart.astrum.security.config;

import org.globalart.astrum.security.config.memory.MemoryCPrivilege.MemoryCPrivilegeBuilder;
import org.globalart.astrum.security.config.memory.MemoryCRole;

/**
 * @since 3.0
 */
public class InitialSecurityConfiguration
{
  public static MemorySecurityConfiguration getConfiguration() {
    return new MemorySecurityConfiguration()
        .withPrivileges(
            new MemoryCPrivilegeBuilder("1-test")
                .type("method")
                .name("1-test")
                .description("")
                .property("method", "read")
                .property("permission", "/some/path/")
                .build(),
            new MemoryCPrivilegeBuilder("2-test")
                .type("method")
                .name("2-test")
                .description("")
                .property("method", "read")
                .property("permission", "/some/path/")
                .build())
        .withRoles(
            new MemoryCRole()
                .withId("test")
                .withName("test Role")
                .withDescription("Test Role Description")
                .withPrivileges("2-test"));
  }
}
