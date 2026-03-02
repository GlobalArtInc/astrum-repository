
package org.globalart.astrum.repository.internal;

import org.globalart.astrum.security.config.CPrivilege;
import org.globalart.astrum.security.config.SecurityConfiguration;

import org.junit.Before;
import org.junit.Test;

import static java.util.stream.Collectors.toList;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.Matchers.is;

public class RepositoryViewSecurityContributorTest
{
  private RepositoryViewSecurityContributor underTest;

  @Before
  public void setup() {
    underTest = new RepositoryViewSecurityContributor();
  }

  @Test
  public void testGetContribution() {
    SecurityConfiguration config = underTest.getContribution();
    assertThat(config.getUsers().size(), is(0));
    assertThat(config.getUserRoleMappings().size(), is(0));
    assertThat(config.getRoles().size(), is(0));
    assertThat(config.getPrivileges().stream().map(CPrivilege::getId).collect(toList()),
        containsInAnyOrder("as-repository-view-*-*-*", "as-repository-view-*-*-browse", "as-repository-view-*-*-read",
            "as-repository-view-*-*-edit", "as-repository-view-*-*-add", "as-repository-view-*-*-delete"));
  }
}
