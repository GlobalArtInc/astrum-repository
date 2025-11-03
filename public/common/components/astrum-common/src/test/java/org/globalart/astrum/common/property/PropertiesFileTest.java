
package org.globalart.astrum.common.property;

import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.file.Files;
import java.util.Properties;

import org.joda.time.format.DateTimeFormat;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.not;
import static org.junit.Assert.assertTrue;

/**
 * Tests for {@link PropertiesFile}
 */
public class PropertiesFileTest
{
  @Rule
  public TemporaryFolder tmpDir = new TemporaryFolder();

  private File file;

  private PropertiesFile underTest;

  @Before
  public void setUp() throws IOException {
    file = tmpDir.newFile("test.properties");
    underTest = new PropertiesFile(file);
  }

  @Test
  public void testStoreProperties() throws IOException {
    underTest.setProperty("foo", "bar");
    underTest.store();
    assertTrue(file.exists());
    assertThat(Files.readString(file.toPath()).length(), not(is(0)));

    // expect the first "comment" line of the file to contain a Date
    String firstLine = Files.readAllLines(file.toPath()).get(0);
    assertTrue(
        DateTimeFormat.forPattern("yyyy-MM-dd HH:mm:ss,SSSZ").parseDateTime(firstLine.substring(1)).isBeforeNow());

    Properties props = new Properties();
    try (InputStream in = Files.newInputStream(file.toPath())) {
      props.load(in);
    }

    assertThat(props.keySet(), hasSize(1));
    assertThat(props.getProperty("foo"), is("bar"));
  }

  @Test
  public void testStoreProperties_withExplicitComment() throws IOException {
    String comment = "At the hundredth meridian where the great plains begin";
    underTest.store(comment);
    assertTrue(file.exists());
    assertThat(Files.readString(file.toPath()).length(), not(is(0)));

    // expect the first "comment" line of the file to contain the specified message
    String firstLine = Files.readAllLines(file.toPath()).get(0);
    assertThat(firstLine, is("#" + comment));
  }

  @Test
  public void testLoadProperties() throws IOException {
    Properties props = new Properties();
    props.setProperty("foo", "bar");

    try (OutputStream out = Files.newOutputStream(file.toPath())) {
      props.store(out, null);
    }

    underTest.load();
    assertThat(props.keySet(), hasSize(1));
    assertThat(props.getProperty("foo"), is("bar"));
  }
}
