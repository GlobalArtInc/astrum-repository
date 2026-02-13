
package org.globalart.astrum.repository.content.upgrades;

import org.globalart.astrum.kv.upgrade.UpgradeAstrumKeyValueStore;
import org.globalart.astrum.repository.content.tasks.normalize.ReNormalizeComponentVersionTask;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;

@RunWith(MockitoJUnitRunner.Silent.class)
public class ScheduleReNormalizationMigrationStepTest
{
  @Mock
  private UpgradeAstrumKeyValueStore keyValueStore;

  private ScheduleReNormalizationMigrationStep migrationStep;

  @Before
  public void setUp() {
    migrationStep = new ScheduleReNormalizationMigrationStep(keyValueStore);
  }

  @Test
  public void testChecksumIsFixed() {
    assertThat(migrationStep.getChecksum(), is(1));
  }

  @Test
  public void testMigrateRaisesGateForEveryTargetFormatOnly() {
    migrationStep.migrate(null);

    for (String format : ReNormalizeComponentVersionTask.RENORMALIZE_FORMATS) {
      verify(keyValueStore).setBoolean(format + ".renormalization.in_progress", true);
    }
    verify(keyValueStore, never()).setBoolean(eq("maven2.renormalization.in_progress"), any(Boolean.class));
    verify(keyValueStore, never()).setBoolean(eq("docker.renormalization.in_progress"), any(Boolean.class));
  }
}
