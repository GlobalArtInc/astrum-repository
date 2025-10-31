
package org.globalart.astrum.common.io;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.nio.charset.Charset;
import java.util.Arrays;
import java.util.List;

import com.google.common.io.ByteStreams;
import com.google.common.io.Resources;
import org.junit.Test;

import static org.junit.Assert.assertEquals;

/**
 * UT for {@link SanitizingJsonOutputStream}.
 *
 * @since 3.0
 */
public class SanitizingJsonOutputStreamTest

{
  private static final List<String> FIELDS = Arrays.asList("A", "B", "C", "D", "E", "F", "G", "K", "M");

  private static final String REPLACEMENT = "*";

  /**
   * Tests that a sanitizer correctly sanitizes basic content based on field names.
   */
  @Test
  public void testSanitizeContent() throws IOException {
    String input = Resources.toString(Resources.getResource(getClass(), "input.json"), Charset.forName("UTF-8"));
    String output = Resources.toString(Resources.getResource(getClass(), "output.json"), Charset.forName("UTF-8"));

    ByteArrayInputStream is = new ByteArrayInputStream(input.getBytes(Charset.forName("UTF-8")));
    ByteArrayOutputStream os = new ByteArrayOutputStream();
    try (SanitizingJsonOutputStream stream = new SanitizingJsonOutputStream(os, FIELDS, REPLACEMENT)) {
      ByteStreams.copy(is, stream);
    }

    assertEquals(output, os.toString("UTF-8"));
  }
}
