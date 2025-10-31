
package org.globalart.astrum.common.i18n;

import org.junit.Before;
import org.junit.Test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNotNull;

/**
 * Tests for {@link MessageBundle} use.
 */
public class MessageBundleTest
{
  public static interface Messages
      extends MessageBundle
  {
    @DefaultMessage("This is a test")
    String test1();

    @DefaultMessage("s:%s,i:%s")
    String testWithFormat(String a, int b);

    String testMissing();

    Object testInvalid();
  }

  private Messages messages;

  @Before
  public void setUp() throws Exception {
    messages = I18N.create(Messages.class);
    assertNotNull(messages);
  }

  @Test
  public void testDefaultMessage() {
    String msg = messages.test1();
    assertEquals("This is a test", msg);
  }

  @Test
  public void testDefaultMessageWithFormat() {
    String msg = messages.testWithFormat("foo", 1);
    assertEquals("s:foo,i:1", msg);
  }

  @Test
  public void testMissing() {
    String msg = messages.testMissing();
    assertEquals(String.format(I18N.MISSING_MESSAGE_FORMAT, "testMissing"), msg);
  }

  @Test(expected = Error.class)
  public void testInvalid() {
    messages.testInvalid();
  }
}
