
package org.globalart.astrum.repository.internal.blobstore;

import org.globalart.astrum.security.config.CPrivilege;
import org.globalart.astrum.security.config.SecurityConfiguration;

import org.junit.Before;
import org.junit.Test;

import static java.util.stream.Collectors.toList;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.Matchers.is;

public class BlobStoreSecurityContributorTest
{
  private BlobStoreSecurityContributor underTest;

  @Before
  public void setup() {
    underTest = new BlobStoreSecurityContributor();
  }

  @Test
  public void testGetContribution() {
    SecurityConfiguration config = underTest.getContribution();
    assertThat(config.getUsers().size(), is(0));
    assertThat(config.getUserRoleMappings().size(), is(0));
    assertThat(config.getRoles().size(), is(0));
    assertThat(config.getPrivileges().stream().map(CPrivilege::getId).collect(toList()),
        containsInAnyOrder("as-blobstores-all", "as-blobstores-create", "as-blobstores-read", "as-blobstores-update",
            "as-blobstores-delete"));
  }
}
