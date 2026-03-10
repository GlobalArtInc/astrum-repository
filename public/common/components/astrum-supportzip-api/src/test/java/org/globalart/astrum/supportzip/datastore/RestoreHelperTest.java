
package org.globalart.astrum.supportzip.datastore;

import java.io.File;

import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;

import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.CoreMatchers.sameInstance;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.junit.Assert.assertThrows;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * Tests for {@link RestoreHelper}
 */
@RunWith(MockitoJUnitRunner.class)
public class RestoreHelperTest
{
  private static final File DB_DIRECTORY = new File("/tmp/work/db");

  @Mock
  private ApplicationDirectories applicationDirectories;

  @Test
  public void getDbPathReturnsWorkDirectoryAsPath() {
    when(applicationDirectories.getWorkDirectory("db", false)).thenReturn(DB_DIRECTORY);

    RestoreHelper underTest = new RestoreHelper(applicationDirectories);

    assertThat(underTest.getDbPath(), is(DB_DIRECTORY.toPath()));
  }

  @Test
  public void constructorResolvesDbWorkDirectory() {
    when(applicationDirectories.getWorkDirectory("db", false)).thenReturn(DB_DIRECTORY);

    new RestoreHelper(applicationDirectories);

    verify(applicationDirectories).getWorkDirectory("db", false);
  }

  @Test
  public void constructorRejectsNullApplicationDirectories() {
    assertThrows(NullPointerException.class, () -> new RestoreHelper(null));
  }

  @Test
  public void fileSuffixConstantIsJson() {
    assertThat(RestoreHelper.FILE_SUFFIX, is(".json"));
  }

  @Test
  public void getDbPathReturnsSameInstanceOnRepeatedCalls() {
    when(applicationDirectories.getWorkDirectory("db", false)).thenReturn(DB_DIRECTORY);

    RestoreHelper underTest = new RestoreHelper(applicationDirectories);

    assertThat(underTest.getDbPath(), is(sameInstance(underTest.getDbPath())));
  }
}
