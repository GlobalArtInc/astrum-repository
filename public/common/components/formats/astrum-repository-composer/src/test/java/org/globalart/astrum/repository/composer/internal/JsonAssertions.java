package org.globalart.astrum.repository.composer.internal;

import java.io.IOException;

import com.fasterxml.jackson.databind.ObjectMapper;

import static org.junit.Assert.assertTrue;

public final class JsonAssertions
{
  private static final ObjectMapper MAPPER = new ObjectMapper();

  private JsonAssertions() {
  }

  public static void assertEquals(final String expected, final String actual, final boolean strict) throws IOException {
    assertTrue("strict comparison only", strict);
    org.junit.Assert.assertEquals(MAPPER.readTree(expected), MAPPER.readTree(actual));
  }
}
