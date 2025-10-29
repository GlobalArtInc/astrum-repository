
package org.globalart.astrum.common.sequence;

import java.util.concurrent.atomic.AtomicLong;
import java.util.function.LongSupplier;

import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Supplies local {@link AtomicSequence}s.
 *
 * @since 3.14
 */
@Component
@Qualifier("local")
public class LocalAtomicSequenceFactory
    implements AtomicSequenceFactory
{
  @Override
  public AtomicSequence create(final String id, final LongSupplier initialValue) {
    AtomicLong sequence = new AtomicLong(initialValue.getAsLong());
    return () -> sequence.incrementAndGet();
  }
}
