
package org.globalart.astrum.common.math;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

/**
 * Tests for {@link Math2}
 */
public class Math2Test

{
  @Test
  public void testAddClamped() {
    // no overflow or underflow
    assertThat(Math2.addClamped(0L, 0L), is(0L));
    assertThat(Math2.addClamped(Long.MAX_VALUE, 0L), is(Long.MAX_VALUE));
    assertThat(Math2.addClamped(0L, Long.MAX_VALUE), is(Long.MAX_VALUE));
    assertThat(Math2.addClamped(Long.MAX_VALUE, -1L), is(Long.MAX_VALUE - 1L));
    assertThat(Math2.addClamped(-1L, Long.MAX_VALUE), is(Long.MAX_VALUE - 1L));
    assertThat(Math2.addClamped(Long.MIN_VALUE, 0L), is(Long.MIN_VALUE));
    assertThat(Math2.addClamped(0L, Long.MIN_VALUE), is(Long.MIN_VALUE));
    assertThat(Math2.addClamped(Long.MIN_VALUE, 1L), is(Long.MIN_VALUE + 1));
    assertThat(Math2.addClamped(1L, Long.MIN_VALUE), is(Long.MIN_VALUE + 1));
    assertThat(Math2.addClamped(Long.MIN_VALUE, Long.MAX_VALUE), is(-1L));
    assertThat(Math2.addClamped(Long.MAX_VALUE, Long.MIN_VALUE), is(-1L));

    // overflow
    assertThat(Math2.addClamped(Long.MAX_VALUE, 1L), is(Long.MAX_VALUE));
    assertThat(Math2.addClamped(Long.MAX_VALUE, Long.MAX_VALUE), is(Long.MAX_VALUE));
    assertThat(Math2.addClamped(Long.MAX_VALUE - 1L, Long.MAX_VALUE - 1L), is(Long.MAX_VALUE));

    // underflow
    assertThat(Math2.addClamped(Long.MIN_VALUE, -1L), is(Long.MIN_VALUE));
    assertThat(Math2.addClamped(Long.MIN_VALUE, Long.MIN_VALUE), is(Long.MIN_VALUE));
    assertThat(Math2.addClamped(Long.MIN_VALUE + 1L, Long.MIN_VALUE + 1L), is(Long.MIN_VALUE));
  }
}
