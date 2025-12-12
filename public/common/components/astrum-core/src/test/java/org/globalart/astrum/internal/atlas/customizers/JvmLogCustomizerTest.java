
package org.globalart.astrum.internal.atlas.customizers;

import java.io.BufferedReader;
import java.io.File;
import java.io.FileWriter;
import java.io.IOException;
import java.io.InputStreamReader;
import java.time.ZonedDateTime;
import java.util.List;

import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;
import org.globalart.astrum.common.log.LogManager;
import org.globalart.astrum.supportzip.SupportBundle;
import org.globalart.astrum.supportzip.SupportBundle.ContentSource;

import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;
import org.junit.runner.RunWith;

import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.CoreMatchers.equalTo;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.when;

import org.globalart.astrum.bootstrap.entrypoint.configuration.DirectoryHelper;

@RunWith(MockitoJUnitRunner.Silent.class)
public class JvmLogCustomizerTest
{
  @Rule
  public TemporaryFolder temporaryWorkDirectory = new TemporaryFolder();

  @Mock
  private LogManager mockLogManager;

  @Mock
  private ApplicationDirectories mockApplicationDirectories;

  private final DirectoryHelper directoryHelper = new DirectoryHelper();

  private JvmLogCustomizer underTest;

  @Before
  public void setup() throws IOException {
    when(mockApplicationDirectories.getWorkDirectory()).thenReturn(temporaryWorkDirectory.getRoot());
    directoryHelper.mkdir(temporaryWorkDirectory.getRoot(), "log");
    underTest = new JvmLogCustomizer(mockLogManager, mockApplicationDirectories);
  }

  @Test
  public void customizeJvmLog() throws Exception {
    File file = createLogFile();
    when(mockLogManager.getLogFile(anyString())).thenReturn(file);
    try (FileWriter writer = new FileWriter(file)) {
      writer.write("-Dastrum.password=astrum -Dastrum.token=123");
    }
    SupportBundle supportBundle = new SupportBundle();
    underTest.customize(supportBundle);

    List<ContentSource> list = supportBundle.getSources();
    assertThat(list.size(), equalTo(1));

    ContentSource contentSource = list.get(0);
    contentSource.prepare();

    try (BufferedReader reader = new BufferedReader(new InputStreamReader(contentSource.getContent()))) {
      assertThat(reader.readLine(), equalTo("-Dastrum.password=**** -Dastrum.token=****"));
    }
  }

  @Test
  public void customizeJvmLogFallsBackToWorkDir() throws Exception {
    File file = createLogFile();
    when(mockLogManager.getLogFile(anyString())).thenReturn(null);
    try (FileWriter writer = new FileWriter(file)) {
      writer.write("-Dastrum.secret=shh");
    }
    SupportBundle supportBundle = new SupportBundle();
    underTest.customize(supportBundle);

    List<ContentSource> list = supportBundle.getSources();
    assertThat(list.size(), equalTo(1));

    ContentSource contentSource = list.get(0);
    contentSource.prepare();

    try (BufferedReader reader = new BufferedReader(new InputStreamReader(contentSource.getContent()))) {
      assertThat(reader.readLine(), equalTo("-Dastrum.secret=****"));
    }
  }

  private File createLogFile() throws IOException {
    File file = new File(temporaryWorkDirectory.getRoot(), "/log/jvm.log");
    file.createNewFile();
    file.setLastModified(ZonedDateTime.now().minusMinutes(0L).toInstant().toEpochMilli());
    return file;
  }

}
