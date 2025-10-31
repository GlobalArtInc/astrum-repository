
package org.globalart.astrum.common.encoding;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.arrayContaining;
import static org.hamcrest.Matchers.is;

public class EncodingUtilTest
{
  @Test
  public void testUrlEncode() {
    assertThat(EncodingUtil.urlEncode("simple"), is("simple"));
    assertThat(EncodingUtil.urlEncode("sim/ple"), is("sim%2Fple"));
  }

  @Test
  public void testUrlDecode() {
    assertThat(EncodingUtil.urlDecode("simple"), is("simple"));
    assertThat(EncodingUtil.urlDecode("sim%2Fple"), is("sim/ple"));
  }

  @Test
  public void testUrlDecodeArray() {
    assertThat(EncodingUtil.urlDecode("simple", "simple2"), arrayContaining("simple", "simple2"));
    assertThat(EncodingUtil.urlDecode("sim%2Fple", "simple2%2f"), arrayContaining("sim/ple", "simple2/"));
  }
}
