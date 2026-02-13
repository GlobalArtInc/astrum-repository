
package org.globalart.astrum.repository.content.upgrades;

import java.io.File;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.sql.Connection;
import java.sql.Statement;

import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;
import org.globalart.astrum.bootstrap.entrypoint.configuration.DirectoryHelper;
import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.repository.internal.blobstore.BlobStoreConfigurationDAO;
import org.globalart.astrum.repository.internal.blobstore.BlobStoreConfigurationData;
import org.globalart.astrum.testdb.DataSessionRule;

import com.google.common.collect.ImmutableMap;
import org.junit.After;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static java.nio.file.StandardOpenOption.CREATE;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.collection.IsArrayWithSize.arrayWithSize;
import static org.hamcrest.core.Is.is;
import static org.hamcrest.core.IsNull.nullValue;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.blobstore.file.FileBlobStore.BASEDIR;
import static org.globalart.astrum.blobstore.file.FileBlobStore.CONFIG_KEY;
import static org.globalart.astrum.blobstore.file.FileBlobStore.PATH_KEY;
import static org.globalart.astrum.blobstore.quota.BlobStoreQuotaSupport.ROOT_KEY;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;
import static org.globalart.astrum.repository.content.upgrades.DatastoreBlobReconciliationLogMigrator_1_11.BLOBSTORE;
import static org.globalart.astrum.repository.content.upgrades.DatastoreBlobReconciliationLogMigrator_1_11.BLOBSTORE_LOG_PATH;
import static org.globalart.astrum.repository.content.upgrades.DatastoreBlobReconciliationLogMigrator_1_11.RECONCILIATION_DIRECTORY_NAME;
import static org.globalart.astrum.repository.content.upgrades.DatastoreBlobReconciliationLogMigrator_1_11.TABLE_NAME;

@RunWith(MockitoJUnitRunner.Silent.class)
public class DatastoreBlobReconciliationLogMigrator_1_11_Test
{
  private static final String BLOBSTORE_1 = "blobstore-1";

  private static final String BLOBSTORE_2 = "blobstore-2";

  @Rule
  public TemporaryFolder sourceReconciliationLogFolder = new TemporaryFolder();

  @Rule
  public TemporaryFolder destReconciliationLogFolder = new TemporaryFolder();

  @Rule
  public DataSessionRule sessionRule = new DataSessionRule(DEFAULT_DATASTORE_NAME)
      .access(BlobStoreConfigurationDAO.class);

  @Mock
  private ApplicationDirectories appDirs;

  private Path currentReconciliationLogBaseDir;

  private DataSession<?> session;

  private final DirectoryHelper directoryHelper = new DirectoryHelper();

  private DatastoreBlobReconciliationLogMigrator_1_11 underTest;

  @Before
  public void setup() throws Exception {
    currentReconciliationLogBaseDir = Paths.get(sourceReconciliationLogFolder.getRoot().toString(), BLOBSTORE);
    directoryHelper.mkdir(currentReconciliationLogBaseDir.toFile());

    underTest = new DatastoreBlobReconciliationLogMigrator_1_11(appDirs);
    session = sessionRule.openSession(DEFAULT_DATASTORE_NAME);
  }

  @After
  public void cleanup() {
    session.close();
  }

  @Test
  public void shouldDoNothingWhenTableDoesNotExist() throws Exception {
    try (Connection connection = sessionRule.openConnection(DEFAULT_DATASTORE_NAME)) {
      dropTable(connection);
      underTest.migrate(connection);

      verify(appDirs, never()).getWorkDirectory(BLOBSTORE_LOG_PATH);
    }
  }

  @Test
  public void shouldDoNothingWhenNoReconciliationLogs() throws Exception {
    try (Connection connection = sessionRule.openConnection(DEFAULT_DATASTORE_NAME)) {

      underTest.migrate(connection);

      verify(appDirs).getWorkDirectory(BLOBSTORE_LOG_PATH);
    }
  }

  @Test
  public void shouldNotCopyFilesWhenBlobStoreDoesNotExist() throws Exception {
    try (Connection connection = sessionRule.openConnection(DEFAULT_DATASTORE_NAME)) {

      when(appDirs.getWorkDirectory(BLOBSTORE_LOG_PATH)).thenReturn(currentReconciliationLogBaseDir.toFile());
      createExistingReconciliationLogFiles();

      underTest.migrate(connection);

      assertThat(getReconciliationDirectoryPath(destReconciliationLogFolder, BLOBSTORE_1).toFile().listFiles(),
          is(nullValue()));
      assertThat(getReconciliationDirectoryPath(destReconciliationLogFolder, BLOBSTORE_2).toFile().listFiles(),
          is(nullValue()));
    }
  }

  @Test
  public void shouldMoveLogsToAppropriateBlobStore() throws Exception {
    try (Connection connection = sessionRule.openConnection(DEFAULT_DATASTORE_NAME)) {

      BlobStoreConfigurationDAO dao = session.access(BlobStoreConfigurationDAO.class);
      dao.create(aBlobStoreConfiguration(BLOBSTORE_1));
      dao.create(aBlobStoreConfiguration(BLOBSTORE_2));

      session.getTransaction().commit();

      when(appDirs.getWorkDirectory(BLOBSTORE_LOG_PATH)).thenReturn(currentReconciliationLogBaseDir.toFile());
      when(appDirs.getWorkDirectory(BASEDIR)).thenReturn(destReconciliationLogFolder.getRoot());
      createExistingReconciliationLogFiles();

      underTest.migrate(connection);

      Path blobStoreReconciliationDirPath = getReconciliationDirectoryPath(destReconciliationLogFolder, BLOBSTORE_1);

      assertThat(blobStoreReconciliationDirPath.toFile().listFiles(), arrayWithSize(3));
      assertThat(blobStoreReconciliationDirPath.resolve("2022-04-07").toFile().exists(), is(true));
      assertThat(blobStoreReconciliationDirPath.resolve("2022-04-08").toFile().exists(), is(true));
      assertThat(blobStoreReconciliationDirPath.resolve("2022-05-21").toFile().exists(), is(true));

      blobStoreReconciliationDirPath = getReconciliationDirectoryPath(destReconciliationLogFolder, BLOBSTORE_2);
      assertThat(blobStoreReconciliationDirPath.toFile().listFiles(), arrayWithSize(4));
      assertThat(blobStoreReconciliationDirPath.resolve("2022-01-08").toFile().exists(), is(true));
      assertThat(blobStoreReconciliationDirPath.resolve("2022-03-18").toFile().exists(), is(true));
      assertThat(blobStoreReconciliationDirPath.resolve("2022-07-10").toFile().exists(), is(true));
      assertThat(blobStoreReconciliationDirPath.resolve("2022-09-21").toFile().exists(), is(true));

      assertThat(currentReconciliationLogBaseDir.toFile().exists(), is(false));
    }
  }

  private Path getReconciliationDirectoryPath(final TemporaryFolder folder, final String blobStoreName) {
    return folder.getRoot().toPath().resolve(Paths.get(blobStoreName, RECONCILIATION_DIRECTORY_NAME));
  }

  private void createExistingReconciliationLogFiles() throws IOException {
    makeBlobStoreDirectory(BLOBSTORE_1);
    makeBlobStoreDirectory(BLOBSTORE_2);
    writeReconciliationLogFile(BLOBSTORE_1, "2022-04-07");
    writeReconciliationLogFile(BLOBSTORE_1, "2022-04-08");
    writeReconciliationLogFile(BLOBSTORE_1, "2022-05-21");
    writeReconciliationLogFile(BLOBSTORE_2, "2022-01-08");
    writeReconciliationLogFile(BLOBSTORE_2, "2022-03-18");
    writeReconciliationLogFile(BLOBSTORE_2, "2022-07-10");
    writeReconciliationLogFile(BLOBSTORE_2, "2022-09-21");
  }

  private void writeReconciliationLogFile(final String blobStoreName, final String fileName) throws IOException {
    String filePath = Paths.get(BLOBSTORE, blobStoreName, fileName).toString();
    String sampleContent = fileName + " 15:00:01,00000000-0000-0000-0000-000000000001";
    File newFile = sourceReconciliationLogFolder.newFile(filePath);
    Files.write(newFile.toPath(), sampleContent.getBytes(StandardCharsets.UTF_8), CREATE);
  }

  private void makeBlobStoreDirectory(final String blobStoreName) throws IOException {
    directoryHelper.mkdir(Paths.get(currentReconciliationLogBaseDir.toString(), blobStoreName).toFile());
  }

  private BlobStoreConfigurationData aBlobStoreConfiguration(final String path) {
    final BlobStoreConfigurationData config = new BlobStoreConfigurationData();
    config.setName(path);
    config.setType(CONFIG_KEY);
    config.setAttributes(ImmutableMap.of(CONFIG_KEY, ImmutableMap.of(PATH_KEY, path), ROOT_KEY, ImmutableMap.of()));
    return config;
  }

  private void dropTable(final Connection connection) throws Exception {
    Statement stmt = connection.createStatement();
    stmt.executeUpdate("DROP TABLE IF EXISTS " + TABLE_NAME);
  }
}
