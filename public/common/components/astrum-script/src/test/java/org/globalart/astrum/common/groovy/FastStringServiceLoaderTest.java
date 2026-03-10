
package org.globalart.astrum.common.groovy;

import org.apache.groovy.json.internal.FastStringUtils;
import org.junit.Test;

import static org.junit.Assert.assertArrayEquals;
import static org.junit.Assert.assertSame;

public class FastStringServiceLoaderTest
{
  private final FastStringServiceLoader underTest = new FastStringServiceLoader();

  @Test
  public void doStartCompletesWithoutThrowing() throws Exception {
    underTest.doStart();
  }

  @Test
  public void fastStringUtilsStillWorksAfterDoStart() throws Exception {
    underTest.doStart();

    assertArrayEquals(new char[]{'a', 'b'}, FastStringUtils.toCharArray("ab"));
  }

  @Test
  public void doStartIsIdempotentAndCanBeInvokedTwice() throws Exception {
    underTest.doStart();
    underTest.doStart();

    // the observable effect must still hold after a repeated start
    assertArrayEquals(new char[]{'a', 'b'}, FastStringUtils.toCharArray("ab"));
  }

  @Test
  public void doStartRestoresThreadContextClassLoader() throws Exception {
    ClassLoader before = Thread.currentThread().getContextClassLoader();

    underTest.doStart();

    // doStart only mutates the TCCL inside a TcclBlock, which must restore it on exit
    assertSame(before, Thread.currentThread().getContextClassLoader());
  }
}
