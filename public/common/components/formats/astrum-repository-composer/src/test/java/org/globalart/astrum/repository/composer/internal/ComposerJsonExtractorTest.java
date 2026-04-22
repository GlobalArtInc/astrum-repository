
package org.globalart.astrum.repository.composer.internal;

import java.io.InputStream;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.util.Map;

import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;
import org.globalart.astrum.blobstore.api.Blob;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.google.common.io.CharStreams;
import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.repository.composer.internal.JsonAssertions.assertEquals;

@RunWith(MockitoJUnitRunner.Silent.class)
public class ComposerJsonExtractorTest
{
  @Mock
  private Blob blob;

  private ComposerJsonExtractor underTest;

  @Before
  public void setUp() {
    underTest = new ComposerJsonExtractor();
  }

  @Test
  public void extractInfoFromZipballWithJson() throws Exception {
    String expected;
    try (InputStream in = getClass().getResourceAsStream("extractInfoFromZipballWithJson.composer.json")) {
      expected = CharStreams.toString(new InputStreamReader(in, StandardCharsets.UTF_8));
    }
    String actual;
    try (InputStream in = getClass().getResourceAsStream("extractInfoFromZipballWithJson.zip")) {
      when(blob.getInputStream()).thenReturn(in);
      actual = new ObjectMapper().writeValueAsString(underTest.extractFromZip(blob));
    }
    assertEquals(expected, actual, true);
  }

  @Test
  public void extractInfoFromZipballWithJsonComposerArchived() throws Exception {
    String expected;
    try (InputStream in = getClass().getResourceAsStream("extractInfoFromZipballWithJsonComposerArchived.composer.json")) {
      expected = CharStreams.toString(new InputStreamReader(in, StandardCharsets.UTF_8));
    }
    String actual;
    try (InputStream in = getClass().getResourceAsStream("extractInfoFromZipBallWithJsonComposerArchived.zip")) {
      when(blob.getInputStream()).thenReturn(in);
      actual = new ObjectMapper().writeValueAsString(underTest.extractFromZip(blob));
    }
    assertEquals(expected, actual, true);
  }

  @Test
  public void extractInfoFromZipballWithoutJson() throws Exception {
    Map<String, Object> results;
    try (InputStream in = getClass().getResourceAsStream("extractInfoFromZipballWithoutJson.zip")) {
      when(blob.getInputStream()).thenReturn(in);
      results = underTest.extractFromZip(blob);
    }
    assertThat(results.isEmpty(), is(true));
  }
}
