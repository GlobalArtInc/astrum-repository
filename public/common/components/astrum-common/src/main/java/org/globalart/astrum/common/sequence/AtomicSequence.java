
package org.globalart.astrum.common.sequence;

/**
 * Atomic sequence of monotonically increasing numbers.
 *
 * @since 3.14
 */
@FunctionalInterface
public interface AtomicSequence
{
  /**
   * Returns the next number in the sequence.
   */
  long next();
}
