
package org.globalart.astrum.repository.content.internal;

import java.sql.Connection;
import java.sql.SQLException;
import java.sql.Statement;
import java.util.Arrays;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.datastore.mybatis.ContinuationArrayList;
import org.globalart.astrum.kv.GlobalKeyValueStore;
import org.globalart.astrum.kv.AstrumKeyValue;
import org.globalart.astrum.kv.ValueType;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.content.store.ComponentData;
import org.globalart.astrum.repository.content.store.ComponentStore;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.globalart.astrum.repository.content.tasks.normalize.NormalizeComponentVersionTaskDescriptor;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.TaskScheduler;
import org.globalart.astrum.scheduling.UpgradeTaskScheduler;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationUtility;

import org.junit.After;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.MockedStatic;
import org.mockito.Mockito;
import org.mockito.junit.MockitoJUnitRunner;

import static java.lang.String.format;
import static org.mockito.ArgumentMatchers.anyList;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

@RunWith(MockitoJUnitRunner.Silent.class)
public class ComponentNormalizationIntegrityCheckerTest
{
  private static final String MAVEN2_FORMAT = "maven2";

  private static final String RAW_FORMAT = "raw";

  private static final String MAVEN2_TABLE = "maven2_component";

  private static final String RAW_TABLE = "raw_component";

  private static final String COLUMN_NAME = "normalized_version";

  private static final String MAVEN2_INDEX = "idx_maven2_normalized_version";

  private static final String RAW_INDEX = "idx_raw_normalized_version";

  private final String ADD_COLUMN_TO_MAVEN2 =
      format("ALTER TABLE %s ADD COLUMN IF NOT EXISTS %s VARCHAR;", MAVEN2_TABLE, COLUMN_NAME);

  private final String ADD_COLUMN_TO_RAW =
      format("ALTER TABLE %s ADD COLUMN IF NOT EXISTS %s VARCHAR;", RAW_TABLE, COLUMN_NAME);

  private final String ADD_INDEX_TO_MAVEN2 =
      format("CREATE INDEX IF NOT EXISTS %s ON %s (%s)", MAVEN2_INDEX, MAVEN2_TABLE.toUpperCase(), COLUMN_NAME);

  private final String ADD_INDEX_TO_RAW =
      format("CREATE INDEX IF NOT EXISTS %s ON %s (%s)", RAW_INDEX, RAW_TABLE.toUpperCase(), COLUMN_NAME);

  private final Continuation<ComponentData> EMPTY_CONTINUATION =
      new ContinuationArrayList<>();

  private final List<Format> formats = Arrays.asList(new MockFormat(MAVEN2_FORMAT), new MockFormat(RAW_FORMAT));

  @Mock
  private FormatStoreManager mavenStoreManager;

  @Mock
  private FormatStoreManager rawStoreManager;

  @Mock
  private ComponentStore mavenComponentStore;

  @Mock
  private ComponentStore rawComponentStore;

  @Mock
  private TaskScheduler taskScheduler;

  @Mock
  private UpgradeTaskScheduler startupScheduler;

  @Mock
  private Map<String, FormatStoreManager> managersByFormat;

  @Mock
  private GlobalKeyValueStore globalKeyValueStore;

  @Mock
  private Connection connection;

  @Mock
  private Statement statement;

  @Mock
  private DatabaseMigrationUtility databaseMigrationUtility;

  private ComponentNormalizationIntegrityChecker checker;

  private MockedStatic<QualifierUtil> mockedStatic;

  @Before
  public void setUp() throws SQLException {
    mockedStatic = Mockito.mockStatic(QualifierUtil.class);
    when(QualifierUtil.buildQualifierBeanMap(anyList()))
        .thenReturn(Map.of(MAVEN2_FORMAT, mavenStoreManager, RAW_FORMAT, rawStoreManager));

    when(statement.execute(ADD_INDEX_TO_RAW)).thenReturn(true);
    checker = new ComponentNormalizationIntegrityChecker(
        formats,
        taskScheduler,
        startupScheduler,
        List.of(mavenStoreManager, rawStoreManager),
        globalKeyValueStore,
        databaseMigrationUtility);

    when(connection.createStatement()).thenReturn(statement);

    when(mavenStoreManager.componentStore(DEFAULT_DATASTORE_NAME)).thenReturn(mavenComponentStore);
    when(rawStoreManager.componentStore(DEFAULT_DATASTORE_NAME)).thenReturn(rawComponentStore);
  }

  @After
  public void tearDown() {
    mockedStatic.close();
  }

  @Test
  public void checkAndRepairSkipsIfTableIsMissing() throws SQLException {
    mockTableMissing();

    // Should complete without throwing - missing table is skipped gracefully
    checker.checkAndRepair(connection);
  }

  @Test
  public void checkAndRepairAddsMissingColumnAndIndex() throws SQLException {
    mockTableExistsButColumnsNeedCreation();
    mockHasNormalizedData(mavenComponentStore);
    mockHasNormalizedData(rawComponentStore);

    checker.checkAndRepair(connection);

    verify(statement).execute(ADD_COLUMN_TO_MAVEN2);
    verify(statement).execute(ADD_INDEX_TO_MAVEN2);

    verify(statement).execute(ADD_COLUMN_TO_RAW);
    verify(statement).execute(ADD_INDEX_TO_RAW);
  }

  @Test
  public void checkAndRepairSchedulesNecessaryTasks() throws SQLException {
    mockTableAndColumnExists();
    mockHasNormalizedData(mavenComponentStore);
    mockHasUnnormalizedData(rawComponentStore);
    TaskConfiguration normalizedVersionTaskConfiguration = normalizedVersionTaskConfiguration();

    checker.checkAndRepair(connection);

    verify(statement, times(0)).execute(ADD_COLUMN_TO_MAVEN2);
    verify(statement, times(0)).execute(ADD_INDEX_TO_MAVEN2);
    verify(statement, times(0)).execute(ADD_COLUMN_TO_RAW);
    verify(statement, times(0)).execute(ADD_INDEX_TO_RAW);
    verify(globalKeyValueStore).setKey(normalizedVersionUnavailable(RAW_FORMAT));
    verify(startupScheduler).schedule(normalizedVersionTaskConfiguration);
  }

  private void mockTableExistsButColumnsNeedCreation() throws SQLException {
    when(databaseMigrationUtility.tableExists(connection, MAVEN2_TABLE.toUpperCase())).thenReturn(true);
    when(databaseMigrationUtility.tableExists(connection, RAW_TABLE.toUpperCase())).thenReturn(true);
    when(databaseMigrationUtility.columnExists(connection, MAVEN2_TABLE, COLUMN_NAME)).thenReturn(false);
    when(databaseMigrationUtility.columnExists(connection, RAW_TABLE, COLUMN_NAME)).thenReturn(false);
    when(databaseMigrationUtility.indexExists(connection, MAVEN2_INDEX)).thenReturn(false);
    when(databaseMigrationUtility.indexExists(connection, RAW_INDEX)).thenReturn(false);
  }

  private void mockTableMissing() throws SQLException {
    when(databaseMigrationUtility.tableExists(connection, MAVEN2_TABLE.toUpperCase())).thenReturn(false);
    when(databaseMigrationUtility.tableExists(connection, RAW_TABLE.toUpperCase())).thenReturn(false);
    mockHasNormalizedData(mavenComponentStore);
    mockHasNormalizedData(rawComponentStore);
  }

  private void mockTableAndColumnExists() throws SQLException {
    when(databaseMigrationUtility.tableExists(connection, MAVEN2_TABLE.toUpperCase())).thenReturn(true);
    when(databaseMigrationUtility.tableExists(connection, RAW_TABLE.toUpperCase())).thenReturn(true);
    when(databaseMigrationUtility.columnExists(connection, MAVEN2_TABLE.toUpperCase(), COLUMN_NAME)).thenReturn(true);
    when(databaseMigrationUtility.columnExists(connection, RAW_TABLE.toUpperCase(), COLUMN_NAME)).thenReturn(true);
  }

  private void mockHasNormalizedData(ComponentStore componentStore) {
    when(componentStore.browseUnnormalized(1, null)).thenReturn(EMPTY_CONTINUATION);
  }

  private void mockHasUnnormalizedData(ComponentStore componentStore) {
    Continuation<ComponentData> continuationWithASingleElement = new ContinuationArrayList()
    {
      {
        add(new ComponentData());
      }
    };
    when(componentStore.browseUnnormalized(1, null)).thenReturn(continuationWithASingleElement);
  }

  private AstrumKeyValue normalizedVersionUnavailable(final String formatName) {
    AstrumKeyValue kv = new AstrumKeyValue();
    kv.setKey(formatName + ".normalized.version.available");
    kv.setType(ValueType.BOOLEAN);
    kv.setValue(false);
    return kv;
  }

  private TaskConfiguration normalizedVersionTaskConfiguration() {
    TaskConfiguration config = new TaskConfiguration();
    when(taskScheduler.createTaskConfigurationInstance(NormalizeComponentVersionTaskDescriptor.TYPE_ID)).thenReturn(
        config);
    return config;
  }

  private static class MockFormat
      extends Format
  {
    public MockFormat(final String formatName) {
      super(formatName);
    }
  }
}
