
package org.globalart.astrum.audit.internal.store;

import java.util.Optional;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

public class AuditEventMigrationStep_2_115Test
{
  private final AuditEventMigrationStep_2_115 underTest = new AuditEventMigrationStep_2_115();

  @Test
  public void testVersion() {
    Optional<String> version = underTest.version();
    assertThat(version.isPresent(), is(true));
    assertThat(version.get(), is("2.115"));
  }
}
