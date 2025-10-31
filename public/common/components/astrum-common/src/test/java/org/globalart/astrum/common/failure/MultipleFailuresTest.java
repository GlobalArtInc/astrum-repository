
package org.globalart.astrum.common.failure;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.junit.Assert.assertTrue;
import static org.junit.Assert.fail;

/**
 * Tests for {@link MultipleFailures}.
 */
public class MultipleFailuresTest
{
  private MultipleFailures underTest;

  @Before
  public void setUp() throws Exception {
    underTest = new MultipleFailures();
  }

  @Test
  public void propagate_with_no_failures() throws Exception {
    assertTrue(underTest.getFailures().isEmpty());
    underTest.maybePropagate();
  }

  @Test
  public void propagate_single() throws Exception {
    underTest.add(new Exception("TEST"));
    try {
      underTest.maybePropagate("OOPS");
      fail();
    }
    catch (Exception e) {
      Throwable[] suppressed = e.getSuppressed();
      assertThat(suppressed.length, is(1));
      assertThat(suppressed[0].getMessage(), is("TEST"));
    }
  }

  @Test
  public void propagate_multi() throws Exception {
    underTest.add(new Exception("FOO"));
    underTest.add(new Exception("BAR"));
    try {
      underTest.maybePropagate("OOPS");
      fail();
    }
    catch (Exception e) {
      Throwable[] suppressed = e.getSuppressed();
      assertThat(suppressed.length, is(2));
      assertThat(suppressed[0].getMessage(), is("FOO"));
      assertThat(suppressed[1].getMessage(), is("BAR"));
    }
  }
}
