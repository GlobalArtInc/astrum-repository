
package org.globalart.astrum.internal.security;

import org.globalart.astrum.security.config.CPrivilege;
import org.globalart.astrum.security.config.MemorySecurityConfiguration;

import org.junit.Before;
import org.junit.Test;

import static java.util.stream.Collectors.toList;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;

public class AstrumSecurityContributorTest
{
  private AstrumSecurityContributor underTest;

  @Before
  public void setup() {
    underTest = new AstrumSecurityContributor(true);
  }

  @Test
  public void testGetContribution() {
    MemorySecurityConfiguration config = underTest.getContribution();
    assertThat(config.getUsers().size(), is(0));
    assertThat(config.getUserRoleMappings().size(), is(0));
    assertThat(config.getRoles().size(), is(2));
    assertThat(config.getRole("as-admin").getPrivileges(), containsInAnyOrder("as-all"));
    assertThat(config.getRole("as-anonymous").getPrivileges(),
        containsInAnyOrder("as-search-read", "as-healthcheck-read", "as-repository-view-*-*-browse",
            "as-repository-view-*-*-read"));
    assertThat(config.getPrivileges().stream().map(CPrivilege::getId).collect(toList()),
        containsInAnyOrder("as-all",
            "as-component-upload",
            "as-search-read",
            "as-bundles-all", "as-bundles-read",
            "as-settings-all", "as-settings-read", "as-settings-update",
            "as-apikey-all",
            "as-userschangepw",
            "as-users-all", "as-users-create", "as-users-read", "as-users-update", "as-users-delete",
            "as-roles-all", "as-roles-create", "as-roles-read", "as-roles-update", "as-roles-delete",
            "as-privileges-all", "as-privileges-create", "as-privileges-read", "as-privileges-update",
            "as-privileges-delete"));
  }

  @Test
  public void testGetContributionAnonymousDisabled() {
    MemorySecurityConfiguration config = new AstrumSecurityContributor(false).getContribution();
    assertThat(config.getUsers().size(), is(0));
    assertThat(config.getUserRoleMappings().size(), is(0));
    assertThat(config.getRoles().size(), is(1));
    assertThat(config.getRole("as-admin").getPrivileges(), containsInAnyOrder("as-all"));
    assertThat(config.getRole("as-anonymous"), is(nullValue()));
    assertThat(config.getPrivileges().stream().map(CPrivilege::getId).collect(toList()),
        containsInAnyOrder("as-all",
            "as-component-upload",
            "as-search-read",
            "as-bundles-all", "as-bundles-read",
            "as-settings-all", "as-settings-read", "as-settings-update",
            "as-apikey-all",
            "as-userschangepw",
            "as-users-all", "as-users-create", "as-users-read", "as-users-update", "as-users-delete",
            "as-roles-all", "as-roles-create", "as-roles-read", "as-roles-update", "as-roles-delete",
            "as-privileges-all", "as-privileges-create", "as-privileges-read", "as-privileges-update",
            "as-privileges-delete"));
  }
}
