
package org.globalart.astrum.coreui.internal;

import java.io.File;
import java.util.Map;

import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.coreui.internal.WorkDirectoryStateContributor.WORK_DIRECTORY_KEY;

public class WorkDirectoryStateContributorTest
{
  @Test
  public void shouldReturnWorkDirectoryAbsolutePath() {
    ApplicationDirectories directories = mock(ApplicationDirectories.class);
    when(directories.getWorkDirectory()).thenReturn(new File("/astrum-data"));

    WorkDirectoryStateContributor underTest = new WorkDirectoryStateContributor(directories);

    Map<String, Object> state = underTest.getState();

    assertThat(state.get(WORK_DIRECTORY_KEY), is("/astrum-data"));
  }

  @Test
  public void shouldReturnAbsolutePathForRelativeWorkDirectory() {
    ApplicationDirectories directories = mock(ApplicationDirectories.class);
    File workDir = new File("relative/path");
    when(directories.getWorkDirectory()).thenReturn(workDir);

    WorkDirectoryStateContributor underTest = new WorkDirectoryStateContributor(directories);

    Map<String, Object> state = underTest.getState();

    assertThat(state.get(WORK_DIRECTORY_KEY), is(workDir.getAbsolutePath()));
  }
}
