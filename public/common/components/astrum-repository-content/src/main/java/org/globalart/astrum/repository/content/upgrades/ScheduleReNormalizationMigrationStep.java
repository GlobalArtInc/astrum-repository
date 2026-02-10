
package org.globalart.astrum.repository.content.upgrades;

import java.sql.Connection;

import org.globalart.astrum.kv.upgrade.UpgradeAstrumKeyValueStore;
import org.globalart.astrum.repository.content.internal.ReNormalizationTaskLauncher;
import org.globalart.astrum.repository.content.tasks.normalize.ReNormalizeComponentVersionTask;
import org.globalart.astrum.upgrade.datastore.RepeatableDatabaseMigrationStep;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.content.tasks.normalize.ReNormalizeComponentVersionTask.GATE_KEY_FORMAT;
import static org.globalart.astrum.repository.content.tasks.normalize.ReNormalizeComponentVersionTask.RENORMALIZE_FORMATS;

/**
 * Raises the {@link ReNormalizeComponentVersionTask#GATE_KEY_FORMAT} gate for every format affected by the
 * ASTRUM-53066 version-normalizer encoding change. {@link ReNormalizationTaskLauncher} picks up the raised gates
 * on the next startup and submits {@link ReNormalizeComponentVersionTask} to repair the {@code normalized_version}
 * rows already written under the old encoding. The gate keeps retain-N cleanup disabled for those formats (see
 * {@code ProCleanupFeatureCheck}) until the task clears it format-by-format.
 * <p>
 * Deliberately does not touch the database schema, the null-only backfill migration handled by
 * {@code ComponentNormalizedVersionMigrationStep}, or the task scheduler - this step only flips key-value flags.
 * It runs at {@link org.globalart.astrum.common.app.ManagedLifecycle.Phase#UPGRADE}, before the live
 * {@link org.globalart.astrum.scheduling.TaskScheduler} is started, so it must not submit anything itself.
 * <p>
 * Deliberately {@link RepeatableDatabaseMigrationStep repeatable} rather than a numbered
 * {@code DatabaseMigrationStep}, and identified by name + {@link #getChecksum()} rather than a version number.
 * Numbered steps require a globally monotonically increasing version number across every active release line;
 * if 3.95 consumes version 2.173 and a 3.95.x patch later needs its own migration, inserting one means
 * coordinating a version slot with 3.96's range across two branches at once. A repeatable step sidesteps that:
 * Flyway only asks "has a step named {@code ScheduleReNormalizationMigrationStep} with checksum
 * {@link #getChecksum()} already run on this instance?" - so it runs exactly once per instance regardless of
 * which release line applies it:
 * 
 * <pre>
 *   3.94    -&gt; 3.95            checksum not yet recorded -&gt; runs, gates raised, repair fires
 *   3.95    -&gt; 3.95.1 (patch)  checksum already recorded -&gt; skipped
 *   3.95.1  -&gt; 3.97            checksum already recorded -&gt; skipped
 *   3.94    -&gt; 3.97 (direct)   checksum not yet recorded -&gt; runs, gates raised, repair fires
 * </pre>
 * 
 * As long as {@link #getChecksum()} keeps returning the same value, this behaves as a one-shot migration per
 * instance. The only way it runs again is a deliberate checksum bump - that must only be done to re-arm the
 * repair for a newly discovered normalizer encoding change, never as an accidental side effect of an unrelated
 * edit to this class.
 */
@Component
public class ScheduleReNormalizationMigrationStep
    implements RepeatableDatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final UpgradeAstrumKeyValueStore keyValueStore;

  @Autowired
  public ScheduleReNormalizationMigrationStep(final UpgradeAstrumKeyValueStore keyValueStore) {
    this.keyValueStore = checkNotNull(keyValueStore);
  }

  @Override
  public Integer getChecksum() {
    // never change this value except to deliberately re-arm the repair for a new normalizer encoding change -
    // see the class javadoc for why a fixed checksum is what makes this a safe one-shot-per-instance migration
    return 1;
  }

  @Override
  public void migrate(final Connection connection) {
    log.info("raising re-normalization gate for {}", RENORMALIZE_FORMATS);
    RENORMALIZE_FORMATS.forEach(format -> keyValueStore.setBoolean(GATE_KEY_FORMAT.formatted(format), true));
  }
}
