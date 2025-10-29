
package org.globalart.astrum.common.sequence;

import com.google.common.base.Preconditions;

/**
 * Simple handy class to subclass when you want to wrap another {@link NumberSequence}.
 */
public abstract class NumberSequenceWrapper
    implements NumberSequence
{
  private final NumberSequence delegate;

  public NumberSequenceWrapper(final NumberSequence delegate) {
    this.delegate = Preconditions.checkNotNull(delegate);
  }

  @Override
  public long next() {
    return delegate.next();
  }

  @Override
  public long prev() {
    return delegate.prev();
  }

  @Override
  public long peek() {
    return delegate.peek();
  }

  @Override
  public void reset() {
    delegate.reset();
  }
}
