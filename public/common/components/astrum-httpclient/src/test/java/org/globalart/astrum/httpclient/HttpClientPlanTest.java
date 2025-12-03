
package org.globalart.astrum.httpclient;

import org.junit.Before;
import org.junit.Test;

import static org.junit.Assert.assertEquals;

/**
 * Unit tests for {@link HttpClientPlan}.
 */
public class HttpClientPlanTest
{

  private HttpClientPlan underTest;

  @Before
  public void setUp() throws Exception {
    underTest = new HttpClientPlan();
  }

  @Test
  public void testGetUserAgentNull() {
    testGetUserAgent(null, null);
  }

  @Test
  public void testGetUserAgentNoSuffix() {
    testGetUserAgent("Agent of Astrum", null);
  }

  @Test
  public void testGetUserAgentWithSuffix() {
    testGetUserAgent("Agent of Astrum", "Some suffix");
  }

  private void testGetUserAgent(String base, String suffix) {
    underTest.setUserAgentBase(base);
    String expected = base;
    if (suffix != null) {
      underTest.setUserAgentSuffix(suffix);
      expected = base + " " + suffix;
    }

    // Execute
    String returned = underTest.getUserAgent();

    // Verify
    assertEquals(expected, returned);
  }

}
