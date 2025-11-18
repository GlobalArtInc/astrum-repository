
package org.globalart.astrum.internal.wonderland;

import org.globalart.astrum.security.config.CPrivilege;
import org.globalart.astrum.security.config.MemorySecurityConfiguration;

import org.junit.Before;
import org.junit.Test;

import static java.util.stream.Collectors.toList;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.Matchers.is;

public class WonderlandSecurityContributorTest
{
  private WonderlandSecurityContributor underTest;

  @Before
  public void setup() {
    underTest = new WonderlandSecurityContributor();
  }

  @Test
  public void testGetContribution() {
    MemorySecurityConfiguration config = underTest.getContribution();
    assertThat(config.getUsers().size(), is(0));
    assertThat(config.getUserRoleMappings().size(), is(0));
    assertThat(config.getRoles().size(), is(0));
    assertThat(config.getPrivileges().stream().map(CPrivilege::getId).collect(toList()),
        containsInAnyOrder("as-wonderland-all"));
  }
}
