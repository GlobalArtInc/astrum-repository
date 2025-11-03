
package org.globalart.astrum.common.thread;

import java.security.SecureClassLoader;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

/**
 * Tests for {@link TcclBlock}.
 */
public class TcclBlockTest

{
  @Test
  public void testBeginAndRestoreClassLoader() {
    Thread thread = Thread.currentThread();
    ClassLoader original = thread.getContextClassLoader();
    ClassLoader classLoader = new SecureClassLoader(getClass().getClassLoader())
    {
    };

    try (TcclBlock tccl = TcclBlock.begin(classLoader)) {
      assertThat(thread.getContextClassLoader(), is(classLoader));
    }
    assertThat(thread.getContextClassLoader(), is(original));
  }
}
