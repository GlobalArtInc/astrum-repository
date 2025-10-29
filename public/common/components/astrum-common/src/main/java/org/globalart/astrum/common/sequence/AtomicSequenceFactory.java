
package org.globalart.astrum.common.sequence;

import java.util.function.LongSupplier;

/**
 * Supplies {@link AtomicSequence}s.
 *
 * @since 3.14
 */
public interface AtomicSequenceFactory
{
  /**
   * Creates a new {@link AtomicSequence}.
   *
   * @param id identifies this sequence for debugging purposes
   * @param initialValue the initial value for this sequence
   */
  AtomicSequence create(String id, LongSupplier initialValue);
}
