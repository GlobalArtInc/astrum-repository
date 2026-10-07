
package org.globalart.astrum.self.hosted.datastore;

import java.io.File;
import java.io.FileNotFoundException;
import java.io.FileOutputStream;
import java.io.IOException;
import java.util.HashMap;
import java.util.Map;
import java.util.zip.ZipEntry;
import java.util.zip.ZipOutputStream;

import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;
import org.globalart.astrum.datastore.api.DataStoreConfiguration;

import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;
import org.mockito.Mock;

import static org.hamcrest.Matchers.arrayContaining;
import static org.hamcrest.Matchers.arrayWithSize;
import static org.junit.Assert.assertFalse;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.junit.Assert.assertTrue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

/**
 * @since 3.21
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class DataStoreRestorerLocalImplTest
{

  private static final String RESTORE_FROM_BACKUP = "restore-from-backup";

  private static final String DB = "db";

  @Rule
  public TemporaryFolder temporaryFolder = new TemporaryFolder();

  @Mock
  private ApplicationDirectories directories;

  @Mock
  private DataStoreConfiguration dataStoreConfiguration;

  private DataStoreRestorerLocalImpl underTest;

  private File workDirectory;

  @Before
  public void setup() throws IOException {
    workDirectory = temporaryFolder.newFolder();
    when(directories.getWorkDirectory(any())).thenAnswer(i -> new File(workDirectory, (String) i.getArguments()[0]));
    when(directories.getWorkDirectory(DB, false))
        .thenReturn(new File(workDirectory, DB));
    when(dataStoreConfiguration.getName()).thenReturn("foo");

    underTest = new DataStoreRestorerLocalImpl(directories);
  }

  @Test
  public void testMaybeRestore() throws IOException {
    makeWorkDirectory(RESTORE_FROM_BACKUP);
    createBackup("foo");

    assertTrue(underTest.maybeRestore(dataStoreConfiguration));

    // check no file was unzipped
    File dbDir = directories.getWorkDirectory(DB);
    assertThat(dbDir.list(), arrayContaining("foo.mv.db"));
  }

  @Test
  public void testMaybeRestore_newInstall() {
    when(dataStoreConfiguration.getName()).thenReturn("config");
    assertFalse(underTest.maybeRestore(dataStoreConfiguration));
  }

  @Test
  public void testMaybeRestore_existingDb() throws IOException {
    makeWorkDirectory(DB);
    directories.getWorkDirectory("db/foo.mv.db").createNewFile();
    makeWorkDirectory(RESTORE_FROM_BACKUP);
    createBackup("foo");

    assertFalse(underTest.maybeRestore(dataStoreConfiguration));

    // check no file was unzipped
    File dbDir = directories.getWorkDirectory(DB);
    assertThat(dbDir.listFiles(), arrayWithSize(1));
  }

  @Test
  public void testMaybeRestore_noRestoreDirectory() throws IOException {
    makeWorkDirectory(RESTORE_FROM_BACKUP);
    assertFalse(underTest.maybeRestore(dataStoreConfiguration));
    File dbDir = directories.getWorkDirectory(DB);
    assertFalse(dbDir.exists());
  }

  @Test
  public void testMaybeRestore_postgresqlConfigured() throws IOException {
    // Setup PostgreSQL configuration
    Map<String, String> attributes = new HashMap<>();
    attributes.put("jdbcUrl", "jdbc:postgresql://localhost:5432/astrum");
    when(dataStoreConfiguration.getAttributes()).thenReturn(attributes);

    makeWorkDirectory(RESTORE_FROM_BACKUP);
    createBackup("foo");

    assertFalse(underTest.maybeRestore(dataStoreConfiguration));

    // Verify no db directory was created
    File dbDir = directories.getWorkDirectory(DB);
    assertFalse(dbDir.exists());
  }

  @Test
  public void testMaybeRestore_postgresqlWithInvalidBackup() throws IOException {
    Map<String, String> attributes = new HashMap<>();
    attributes.put("jdbcUrl", "jdbc:postgresql://localhost:5432/astrum");
    when(dataStoreConfiguration.getAttributes()).thenReturn(attributes);

    makeWorkDirectory(RESTORE_FROM_BACKUP);
    new File(directories.getWorkDirectory(RESTORE_FROM_BACKUP), "invalidDir").mkdirs();

    assertFalse(underTest.maybeRestore(dataStoreConfiguration));

    // Verify no db directory was created
    File dbDir = directories.getWorkDirectory(DB);
    assertFalse(dbDir.exists());
  }

  @Test
  public void testMaybeRestore_h2ExplicitlyConfigured() throws IOException {
    Map<String, String> attributes = new HashMap<>();
    attributes.put("jdbcUrl", "jdbc:h2:file:./globalart-work/astrum3/db/astrum");
    when(dataStoreConfiguration.getAttributes()).thenReturn(attributes);

    makeWorkDirectory(RESTORE_FROM_BACKUP);
    createBackup("foo");

    assertTrue(underTest.maybeRestore(dataStoreConfiguration));

    // Verify db was restored
    File dbDir = directories.getWorkDirectory(DB);
    assertThat(dbDir.list(), arrayContaining("foo.mv.db"));
  }

  @Test
  public void testMaybeRestore_noAttributesDefaultsToH2() throws IOException {
    when(dataStoreConfiguration.getAttributes()).thenReturn(null);

    makeWorkDirectory(RESTORE_FROM_BACKUP);
    createBackup("foo");

    assertTrue(underTest.maybeRestore(dataStoreConfiguration));

    // Verify db was restored
    File dbDir = directories.getWorkDirectory(DB);
    assertThat(dbDir.list(), arrayContaining("foo.mv.db"));
  }

  private File makeWorkDirectory(final String path) throws IOException {
    File dir = directories.getWorkDirectory(path);
    dir.mkdirs();
    return dir;
  }

  private void createBackup(final String name) throws FileNotFoundException, IOException {
    File restoreDirectory = makeWorkDirectory(RESTORE_FROM_BACKUP);
    restoreDirectory.mkdirs();
    File zip = new File(restoreDirectory, name);
    try (ZipOutputStream out = new ZipOutputStream(new FileOutputStream(zip))) {
      ZipEntry entry = new ZipEntry(name.concat(".mv.db"));
      out.putNextEntry(entry);
      out.write(name.getBytes());
      out.closeEntry();
    }
  }
}
