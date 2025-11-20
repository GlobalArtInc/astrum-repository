
package org.globalart.astrum.outbound.context;

import org.junit.Before;
import org.junit.After;
import org.junit.Test;

import static org.junit.Assert.*;

public class OutboundRequestContextTest
{

  @Before
  public void setUp() {
    // Clear the context before each test to ensure isolation
    OutboundRequestContext.remove();
  }

  @After
  public void tearDown() {
    // Clear the context after each test to avoid interference
    OutboundRequestContext.remove();
  }

  @Test
  public void testSetAndGetDownloadTime() {
    long downloadTime = 12345L;
    OutboundRequestContext.setDownloadTime(downloadTime);

    Long retrievedTime = OutboundRequestContext.getDownloadTime();
    assertNotNull("Download time should not be null", retrievedTime);
    assertEquals("Download time should match the set value", downloadTime, retrievedTime.longValue());
  }

  @Test
  public void testSetAndGetFormattedString() {
    String formattedString = "Test formatted string";
    OutboundRequestContext.setFormattedString(formattedString);

    String retrievedString = OutboundRequestContext.getFormattedString();
    assertNotNull("Formatted string should not be null", retrievedString);
    assertEquals("Formatted string should match the set value", formattedString, retrievedString);
  }

  @Test
  public void testRemoveClearsContext() {
    OutboundRequestContext.setDownloadTime(12345L);
    OutboundRequestContext.setFormattedString("Test formatted string");

    OutboundRequestContext.remove();

    assertNull("Download time should be null after remove", OutboundRequestContext.getDownloadTime());
    assertNull("Formatted string should be null after remove", OutboundRequestContext.getFormattedString());
  }

  @Test
  public void testGetContextMapSize() {
    assertEquals("Initial context map size should be 0", 0, OutboundRequestContext.getContextMapSize());

    OutboundRequestContext.setDownloadTime(12345L);
    assertEquals("Context map size should be 1 after setting download time", 1,
        OutboundRequestContext.getContextMapSize());

    OutboundRequestContext.setFormattedString("Test formatted string");
    assertEquals("Context map size should be 2 after setting formatted string", 2,
        OutboundRequestContext.getContextMapSize());
  }
}
