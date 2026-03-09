
package org.globalart.astrum.script.plugin.internal.security;

import org.globalart.astrum.security.config.CPrivilege;
import org.globalart.astrum.security.config.SecurityConfiguration;

import org.junit.Before;
import org.junit.Test;

import static java.util.stream.Collectors.toList;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.Matchers.is;

public class ScriptSecurityContributorTest
{
  private ScriptSecurityContributor underTest;

  @Before
  public void setup() {
    underTest = new ScriptSecurityContributor();
  }

  @Test
  public void testGetContribution() {
    SecurityConfiguration config = underTest.getContribution();
    assertThat(config.getUsers().size(), is(0));
    assertThat(config.getUserRoleMappings().size(), is(0));
    assertThat(config.getRoles().size(), is(0));
    assertThat(config.getPrivileges().stream().map(CPrivilege::getId).collect(toList()),
        containsInAnyOrder("as-script-*-*", "as-script-*-browse", "as-script-*-read", "as-script-*-edit",
            "as-script-*-add", "as-script-*-delete", "as-script-*-run"));
  }
}
