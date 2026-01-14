
package org.globalart.astrum.audit.internal.security;

import org.globalart.astrum.security.config.CPrivilege;
import org.globalart.astrum.security.config.MemorySecurityConfiguration;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;

/**
 * Tests for {@link AuditSecurityContributor}.
 */
public class AuditSecurityContributorTest
{
  private AuditSecurityContributor underTest;

  @Before
  public void setUp() {
    underTest = new AuditSecurityContributor();
  }

  @Test
  public void testContribution() {
    MemorySecurityConfiguration config = underTest.getContribution();

    assertThat(config, notNullValue());
    assertThat(config.getPrivileges(), hasSize(2));
  }

  @Test
  public void testReadPrivilege() {
    MemorySecurityConfiguration config = underTest.getContribution();

    CPrivilege readPrivilege = config.getPrivileges()
        .stream()
        .filter(p -> p.getId().equals(AuditSecurityContributor.AUDIT_READ_PRIV_ID))
        .findFirst()
        .orElse(null);

    assertThat(readPrivilege, notNullValue());
    assertThat(readPrivilege.getId(), is("as-audit-read"));
    assertThat(readPrivilege.getType(), is("application"));
    assertThat(readPrivilege.getDescription(), containsString("Read"));
    assertThat(readPrivilege.getDescription(), containsString("Audit"));
    assertThat(readPrivilege.getProperties().get("domain"), is("audit"));
    assertThat(readPrivilege.getProperties().get("actions"), is("read"));
  }

  @Test
  public void testAllPrivilege() {
    MemorySecurityConfiguration config = underTest.getContribution();

    CPrivilege allPrivilege = config.getPrivileges()
        .stream()
        .filter(p -> p.getId().equals(AuditSecurityContributor.AUDIT_ALL_PRIV_ID))
        .findFirst()
        .orElse(null);

    assertThat(allPrivilege, notNullValue());
    assertThat(allPrivilege.getId(), is("as-audit-all"));
    assertThat(allPrivilege.getType(), is("application"));
    assertThat(allPrivilege.getDescription(), containsString("All permissions"));
    assertThat(allPrivilege.getDescription(), containsString("Audit"));
    assertThat(allPrivilege.getProperties().get("domain"), is("audit"));
    assertThat(allPrivilege.getProperties().get("actions"), is("*"));
  }
}
