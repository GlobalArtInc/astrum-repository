
package org.globalart.astrum.datastore.h2.task;

import org.globalart.astrum.common.db.DatabaseCheck;

import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.when;

/**
 * Tests for {@link H2BackupTaskDescriptor}.
 *
 * <p>
 * Regression coverage for ASTRUM-39391: the H2 backup task descriptor must only be
 * visible and exposed on H2-backed installations.
 * </p>
 */
@RunWith(MockitoJUnitRunner.class)
public class H2BackupTaskDescriptorTest
{
  @Mock
  private DatabaseCheck databaseCheck;

  @Test
  public void isVisibleAndExposedOnH2() {
    when(databaseCheck.isPostgresql()).thenReturn(false);

    H2BackupTaskDescriptor descriptor = new H2BackupTaskDescriptor(databaseCheck);

    assertThat(descriptor.isVisible(), is(true));
    assertThat(descriptor.isExposed(), is(true));
  }

  @Test
  public void isHiddenAndNotExposedOnPostgres() {
    when(databaseCheck.isPostgresql()).thenReturn(true);

    H2BackupTaskDescriptor descriptor = new H2BackupTaskDescriptor(databaseCheck);

    assertThat(descriptor.isVisible(), is(false));
    assertThat(descriptor.isExposed(), is(false));
  }
}
