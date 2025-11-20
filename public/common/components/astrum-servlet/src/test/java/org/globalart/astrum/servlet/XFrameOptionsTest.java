
package org.globalart.astrum.servlet;

import java.util.Arrays;
import java.util.Collection;

import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.Parameterized;
import org.junit.runners.Parameterized.Parameter;
import org.junit.runners.Parameterized.Parameters;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

/**
 * @since 3.11
 */
@RunWith(Parameterized.class)
public class XFrameOptionsTest
{
  @Parameters
  public static Collection<Object[]> data() {
    return Arrays.asList(new Object[][]{{"/foo/", true, "DENY"}, {"/foo/", false, "SAMEORIGIN"},
        {"/swagger-ui/", true, "SAMEORIGIN"}, {"/COPYRIGHT.html", true, "SAMEORIGIN"},
        {"/OC-LICENSE.html", true, "SAMEORIGIN"}, {"/PRO-LICENSE.html", true, "SAMEORIGIN"},
        {"/static/healthcheck-tos.html", true, "SAMEORIGIN"}});
  }

  @Parameter
  public String url;

  @Parameter(1)
  public boolean deny;

  @Parameter(2)
  public String expected;

  @Test
  public void testGetValueForPath() {
    XFrameOptions xFrameOptions = new XFrameOptions(deny);
    assertThat(xFrameOptions.getValueForPath(url), is(expected));
  }
}
