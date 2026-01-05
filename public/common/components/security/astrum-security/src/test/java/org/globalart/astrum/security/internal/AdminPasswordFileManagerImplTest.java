
package org.globalart.astrum.security.internal;

import java.io.File;
import java.nio.file.Files;

import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;

import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class AdminPasswordFileManagerImplTest
{
  private AdminPasswordFileManagerImpl underTest;

  @Mock
  private ApplicationDirectories applicationDirectories;

  @Rule
  public TemporaryFolder temporaryFolder = new TemporaryFolder();

  @Before
  public void setup() throws Exception {
    when(applicationDirectories.getWorkDirectory()).thenReturn(temporaryFolder.newFolder("workdir"));
    underTest = new AdminPasswordFileManagerImpl(applicationDirectories);
  }

  @Test
  public void testExists() throws Exception {
    File passwordFile = new File(applicationDirectories.getWorkDirectory(), "admin.password");
    assertThat(underTest.exists(), is(false));
    Files.write(passwordFile.toPath(), "testpass".getBytes("UTF-8"));
    assertThat(underTest.exists(), is(true));
    passwordFile.delete();
    assertThat(underTest.exists(), is(false));
  }

  @Test
  public void testGetPath() {
    File passwordFile = new File(applicationDirectories.getWorkDirectory(), "admin.password");
    assertThat(passwordFile.getAbsolutePath(), is(underTest.getPath()));
  }

  @Test
  public void testWriteFile() throws Exception {
    underTest.writeFile("testpass");
    String storedPassword = new String(
        Files.readAllBytes(new File(applicationDirectories.getWorkDirectory(), "admin.password").toPath()));
    assertThat(storedPassword, is("testpass"));
  }

  @Test
  public void testWriteFile_workdirExists() throws Exception {
    File directory = applicationDirectories.getWorkDirectory();
    directory.mkdirs();
    assertThat(directory.isDirectory(), is(true));
    assertThat(directory.exists(), is(true));

    underTest.writeFile("testpass");

    String storedPassword = new String(
        Files.readAllBytes(new File(applicationDirectories.getWorkDirectory(), "admin.password").toPath()));
    assertThat(storedPassword, is("testpass"));
  }

  @Test
  public void testWriteFile_failure() throws Exception {
    File passwordFile = new File(applicationDirectories.getWorkDirectory(), "admin.password");
    Files.write(passwordFile.toPath(), "testpass".getBytes("UTF-8"));
    passwordFile.setWritable(false);

    assertThat(underTest.writeFile("testpass2"), is(false));
    String storedPassword = new String(
        Files.readAllBytes(new File(applicationDirectories.getWorkDirectory(), "admin.password").toPath()));
    assertThat(storedPassword, is("testpass"));
  }

  @Test
  public void testReadFile() throws Exception {
    File passwordFile = new File(applicationDirectories.getWorkDirectory(), "admin.password");
    Files.write(passwordFile.toPath(), "testpass".getBytes("UTF-8"));
    assertThat(underTest.readFile(), is("testpass"));
  }

  @Test
  public void testRemoveFile() throws Exception {
    File passwordFile = new File(applicationDirectories.getWorkDirectory(), "admin.password");
    Files.write(passwordFile.toPath(), "testpass".getBytes("UTF-8"));
    underTest.removeFile();
    assertThat(passwordFile.exists(), is(false));
  }
}
