
package org.globalart.astrum.repository.content.fluent;

import java.util.Collection;

import org.globalart.astrum.common.entity.Continuation;

import com.google.common.base.Function;
import com.google.common.collect.ForwardingCollection;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.collect.Collections2.transform;

/**
 * Fluent {@link Continuation}s.
 *
 * @since 3.24
 */
public class FluentContinuation<E, T>
    extends ForwardingCollection<E>
    implements Continuation<E>
{
  private final Continuation<T> continuation;

  private final Collection<E> fluentCollection;

  public FluentContinuation(final Continuation<T> continuation, final Function<T, E> toFluent) {
    this.continuation = checkNotNull(continuation);
    this.fluentCollection = transform(continuation, toFluent);
  }

  @Override
  protected Collection<E> delegate() {
    return fluentCollection;
  }

  @Override
  public String nextContinuationToken() {
    return continuation.nextContinuationToken();
  }
}
