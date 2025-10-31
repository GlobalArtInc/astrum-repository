
package org.globalart.astrum.common.i18n;

import org.junit.Before;
import org.junit.Test;

import static org.junit.Assert.assertEquals;

/**
 * Tests for {@link ResourceBundleMessageSource}.
 */
public class ResourceBundleMessageSourceTest
{
  private MessageSource messages;

  @Before
  public void setUp() {
    messages = new ResourceBundleMessageSource(getClass());
  }

  @Test
  public void testLoadAndGetMessage() {
    String a = messages.getMessage("a");
    assertEquals("1", a);

    String b = messages.getMessage("b");
    assertEquals("2", b);

    String c = messages.getMessage("c");
    assertEquals("3", c);

    String f = messages.format("f", a, b, c);
    assertEquals("1 2 3", f);
  }

  @Test
  public void testMissingResource() throws Exception {
    try {
      messages.getMessage("no-such-code");
    }
    catch (ResourceNotFoundException e) {
      // ignore
    }
  }

  @Test
  public void testMissingResourceWithDefault() throws Exception {
    String msg = messages.getMessage("no-such-code", "foo");
    assertEquals("foo", msg);
  }
}
