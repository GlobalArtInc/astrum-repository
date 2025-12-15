
package org.globalart.astrum.internal.capability;

import org.globalart.astrum.security.config.CPrivilege;
import org.globalart.astrum.security.config.SecurityConfiguration;

import org.junit.Before;
import org.junit.Test;

import static java.util.stream.Collectors.toList;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.Matchers.is;

public class CapabilitiesSecurityContributorTest
{
  private CapabilitiesSecurityContributor underTest;

  @Before
  public void setup() {
    underTest = new CapabilitiesSecurityContributor();
  }

  @Test
  public void testGetContribution() {
    SecurityConfiguration config = underTest.getContribution();
    assertThat(config.getUsers().size(), is(0));
    assertThat(config.getUserRoleMappings().size(), is(0));
    assertThat(config.getRoles().size(), is(0));
    assertThat(config.getPrivileges().stream().map(CPrivilege::getId).collect(toList()),
        containsInAnyOrder("as-capabilities-all", "as-capabilities-create", "as-capabilities-read",
            "as-capabilities-update", "as-capabilities-delete"));
  }
}
