
package org.globalart.astrum.bootstrap.entrypoint.configuration;

import java.io.File;
import java.io.IOException;

import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

public class ApplicationDirectoriesImplTest
{
  @Rule
  public TemporaryFolder temporaryFolder = TemporaryFolder.builder().assureDeletion().build();

  private File installDir;

  private File workDir;

  private final DirectoryHelper directoryHelper = new DirectoryHelper();

  private ApplicationDirectoriesImpl underTest;

  @Before
  public void setUp() throws Exception {
    installDir = temporaryFolder.newFolder("install");
    workDir = temporaryFolder.newFolder("work");
    underTest =
        new ApplicationDirectoriesImpl(installDir.getAbsolutePath(), workDir.getAbsolutePath(), directoryHelper);
  }

  @Test
  public void ensureTempDir_exists() {
    File dir = underTest.getTemporaryDirectory();
    assertThat(dir, notNullValue());
  }

  @Test
  public void ensureWorkDir_exists() throws IOException {
    File dir = underTest.getWorkDirectory();
    assertThat(dir, notNullValue());
    assertThat(dir.getCanonicalFile(), is(workDir.getCanonicalFile()));
    assertTrue(dir.exists());
  }

  @Test
  public void ensureWorkDir_childExists() {
    File dir = underTest.getWorkDirectory("child");
    assertThat(dir, notNullValue());
    assertTrue(dir.exists());
  }

  @Test
  public void ensureWorkDir_childWithCreateExists() {
    File dir = underTest.getWorkDirectory("child", true);
    assertThat(dir, notNullValue());
    assertTrue(dir.exists());
  }

  @Test
  public void ensureWorkDir_childNoCreateNotExists() {
    File dir = underTest.getWorkDirectory("child", false);
    assertThat(dir, notNullValue());
    assertFalse(dir.exists());
  }

  @Test
  public void ensureWorkDir_referencesGlobalArtWorkFolderUnlessAbsolute() throws IOException {
    File tempDir = temporaryFolder.newFolder("temp");

    File relative = underTest.getWorkDirectory(".");
    File absolute = underTest.getWorkDirectory(tempDir.getAbsolutePath());
    assertThat(relative.getCanonicalFile(), equalTo(workDir.getCanonicalFile()));
    assertThat(absolute.getCanonicalFile(), equalTo(tempDir.getCanonicalFile()));
  }

  @Test
  public void testWithNoWorkDir() {
    File root = temporaryFolder.getRoot();
    workDir = new File(root, "does-not-exist");
    underTest =
        new ApplicationDirectoriesImpl(installDir.getAbsolutePath(), workDir.getAbsolutePath(), directoryHelper);

    // prior to fix, this would throw an NPE
  }
}
