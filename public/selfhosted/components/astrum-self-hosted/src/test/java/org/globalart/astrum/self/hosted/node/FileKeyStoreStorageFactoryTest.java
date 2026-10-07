
package org.globalart.astrum.self.hosted.node;

import java.io.File;

import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;
import org.globalart.astrum.ssl.spi.KeyStoreStorage;

import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.instanceOf;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class FileKeyStoreStorageFactoryTest
{
  @Rule
  public TemporaryFolder temporaryFolder = new TemporaryFolder();

  private File keyStoreDir;

  @Mock
  private ApplicationDirectories appDirs;

  private FileKeyStoreStorageFactory storageManager;

  @Before
  public void setUp() throws Exception {
    keyStoreDir = temporaryFolder.newFolder("keystores");
    when(appDirs.getWorkDirectory("keystores")).thenReturn(keyStoreDir);
    storageManager = new FileKeyStoreStorageFactory(appDirs);
  }

  @Test
  public void testCreateStorage() {
    KeyStoreStorage storage = storageManager.create("test.ks");
    assertThat(storage, is(instanceOf(FileKeyStoreStorage.class)));
    assertThat(((FileKeyStoreStorage) storage).getKeyStoreFile(), is(new File(keyStoreDir, "node/test.ks")));
  }
}
