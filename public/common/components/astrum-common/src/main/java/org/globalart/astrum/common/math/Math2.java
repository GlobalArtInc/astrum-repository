
package org.globalart.astrum.common.math;

import static java.lang.Math.addExact;
import static java.lang.Math.max;

/**
 * Math helpers.
 *
 * @since 3.14
 */
public final class Math2
{
  private Math2() {
    // empty
  }

  /**
   * Adds numbers, clamping at the maximum or minimum value on overflow.
   */
  public static long addClamped(final long a, final long b) {
    try {
      return addExact(a, b);
    }
    catch (ArithmeticException e) { // NOSONAR
      if (max(a, b) > 0) {
        return Long.MAX_VALUE;
      }
      else {
        return Long.MIN_VALUE;
      }
    }
  }
}
