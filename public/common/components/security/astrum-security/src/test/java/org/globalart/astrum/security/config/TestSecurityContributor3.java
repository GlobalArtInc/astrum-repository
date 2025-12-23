
package org.globalart.astrum.security.config;

import org.globalart.astrum.security.config.memory.MemoryCPrivilege.MemoryCPrivilegeBuilder;
import org.globalart.astrum.security.config.memory.MemoryCRole;

public class TestSecurityContributor3
    implements SecurityContributor
{
  private static int instanceCount = 1;

  private String privId = "priv-" + instanceCount++;

  public String getId() {
    return privId;
  }

  @Override
  public MemorySecurityConfiguration getContribution() {
    return new MemorySecurityConfiguration()
        .withPrivileges(
            new MemoryCPrivilegeBuilder("4-test-" + privId)
                .type("method")
                .name("4-test")
                .description("")
                .property("method", "read")
                .property("permission", "/some/path4/")
                .build(),
            new MemoryCPrivilegeBuilder("5-test-" + privId)
                .type("method")
                .name("5-test")
                .description("")
                .property("method", "read")
                .property("permission", "/some/path5/")
                .build(),
            new MemoryCPrivilegeBuilder("6-test-" + privId)
                .type("method")
                .name("6-test")
                .description("")
                .property("method", "read")
                .property("permission", "/some/path6/")
                .build())
        .withRoles(
            new MemoryCRole()
                .withId("anon")
                .withName("")
                .withDescription("")
                .withPrivileges("4-test")
                .withRoles("other"),
            new MemoryCRole()
                .withId("other")
                .withName("Other Role")
                .withDescription("Other Role Description")
                .withPrivileges("6-test"));
  }
}
