
package org.globalart.astrum.repository.content.internal;

import java.sql.Connection;
import java.sql.SQLException;
import java.sql.Statement;
import java.util.List;
import java.util.Map;
import java.util.Set;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.kv.GlobalKeyValueStore;
import org.globalart.astrum.kv.AstrumKeyValue;
import org.globalart.astrum.kv.ValueType;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.globalart.astrum.repository.content.tasks.normalize.NormalizeComponentVersionTask;
import org.globalart.astrum.repository.content.tasks.normalize.NormalizeComponentVersionTaskDescriptor;
import org.globalart.astrum.scheduling.TaskScheduler;
import org.globalart.astrum.scheduling.UpgradeTaskScheduler;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationUtility;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.lang.String.format;
import static java.util.stream.Collectors.toSet;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;
import org.springframework.stereotype.Component;

/**
 * Check if the {format}_component table has the normalized_version column. If not schedule the
 * NormalizeComponentVersionTask to run after startup is complete.
 */
@Component
public class ComponentNormalizationIntegrityChecker
    implements DatabaseIntegrityChecker
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final String TABLE_NAME = "%s_component";

  private final String COLUMN_NAME = "normalized_version";

  private final String INDEX_NAME = "idx_%s_normalized_version";

  private final String ADD_COLUMN_STATEMENT =
      format("ALTER TABLE %s ADD COLUMN IF NOT EXISTS %s VARCHAR;", TABLE_NAME, COLUMN_NAME);

  private final String ADD_INDEX_STATEMENT =
      "CREATE INDEX IF NOT EXISTS %s ON %s (" + COLUMN_NAME + ")";

  private final List<Format> formats;

  private final TaskScheduler taskScheduler;

  private final UpgradeTaskScheduler startupScheduler;

  private final Map<String, FormatStoreManager> managersByFormat;

  private final GlobalKeyValueStore globalKeyValueStore;

  private final DatabaseMigrationUtility databaseMigrationUtility;

  @Autowired
  public ComponentNormalizationIntegrityChecker(
      final List<Format> formats,
      final TaskScheduler taskScheduler,
      final UpgradeTaskScheduler startupScheduler,
      final List<FormatStoreManager> managersByFormatList,
      final GlobalKeyValueStore globalKeyValueStore,
      final DatabaseMigrationUtility databaseMigrationUtility)
  {
    this.formats = checkNotNull(formats);
    this.taskScheduler = checkNotNull(taskScheduler);
    this.startupScheduler = checkNotNull(startupScheduler);
    this.managersByFormat = QualifierUtil.buildQualifierBeanMap(checkNotNull(managersByFormatList));
    this.globalKeyValueStore = checkNotNull(globalKeyValueStore);
    this.databaseMigrationUtility = checkNotNull(databaseMigrationUtility);
  }

  @Override
  public void checkAndRepair(Connection connection) throws SQLException {
    log.info("validating normalized_version columns");
    alterFormats(connection);

    Set<Format> formatsNeedingNormalization = formats.stream()
        .filter(format -> managersByFormat.containsKey(format.getValue()))
        .filter(format -> !managersByFormat.get(format.getValue())
            .componentStore(DEFAULT_DATASTORE_NAME)
            .browseUnnormalized(1, null)
            .isEmpty())
        .collect(toSet());
    boolean needsNormalization = !formatsNeedingNormalization.isEmpty();

    if (needsNormalization) {
      log.info("Formats detected needing normalization {}", formatsNeedingNormalization);
      formatsNeedingNormalization.forEach(this::markFormatAsNeedingNormalization);
      scheduleTask();
    }
  }

  private void markFormatAsNeedingNormalization(final Format format) {
    AstrumKeyValue kv = new AstrumKeyValue();
    kv.setKey(getFormatKey(format));
    kv.setType(ValueType.BOOLEAN);
    kv.setValue(false);

    globalKeyValueStore.setKey(kv);
  }

  private void scheduleTask() {
    startupScheduler.schedule(taskScheduler.createTaskConfigurationInstance(
        NormalizeComponentVersionTaskDescriptor.TYPE_ID));
  }

  private void alterFormats(final Connection connection) throws SQLException {
    try (Statement alterStmt = connection.createStatement()) {
      for (Format format : formats) {
        alter(connection, alterStmt, format);
      }
    }
  }

  private void alter(
      final Connection connection,
      final Statement alterStatement,
      final Format format) throws SQLException
  {
    String formatName = format.getValue();
    String tableName = format(TABLE_NAME, formatName).toUpperCase();

    if (!databaseMigrationUtility.tableExists(connection, tableName)) {
      log.warn("{} component table not found, skipping normalization check", formatName);
      return;
    }

    if (!databaseMigrationUtility.columnExists(connection, tableName, COLUMN_NAME)) {
      log.info("adding missing column '{}' to {} format", COLUMN_NAME, formatName);
      alterStatement.execute(format(ADD_COLUMN_STATEMENT, formatName));

      if (!databaseMigrationUtility.indexExists(connection, format(INDEX_NAME, formatName))) {
        log.info("adding missing index '{}' to {} format", format(INDEX_NAME, formatName), formatName);
        alterStatement.execute(format(ADD_INDEX_STATEMENT, format(INDEX_NAME, formatName), tableName));
      }
    }
  }

  private String getFormatKey(final Format format) {
    return format(NormalizeComponentVersionTask.KEY_FORMAT, format.getValue());
  }
}
