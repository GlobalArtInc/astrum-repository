
package org.globalart.astrum.audit.internal.store;

import java.util.Optional;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

public class AuditEventsContextIndexMigrationStep_2_116Test
{
  private final AuditEventsContextIndexMigrationStep_2_116 underTest =
      new AuditEventsContextIndexMigrationStep_2_116();

  @Test
  public void testVersion() {
    Optional<String> version = underTest.version();
    assertThat(version.isPresent(), is(true));
    assertThat(version.get(), is("2.116"));
  }
}
