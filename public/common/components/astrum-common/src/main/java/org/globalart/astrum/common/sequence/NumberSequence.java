
package org.globalart.astrum.common.sequence;

/**
 * A simple interface that gives you a sequence of numbers.
 *
 * That might be simple natural numbers sequence, but anything else too.
 */
public interface NumberSequence
{
  /**
   * Returns the next number in sequence and advances the sequence.
   */
  long next();

  /**
   * Returns the previous number in sequence and digresses the sequence.
   */
  long prev();

  /**
   * Returns the next number in sequence without advancing the sequence. This method will return always the same
   * number unless method {@code next()} is called.
   */
  long peek();

  /**
   * Resets the sequence.
   */
  void reset();
}
