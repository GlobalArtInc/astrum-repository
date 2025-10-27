
package org.globalart.astrum.common.io;

import java.util.Map.Entry;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ConcurrentMap;
import java.util.stream.Stream;

import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Supplies local {@link Cooperation} points.
 *
 * @since 3.14
 */
@Component
@Qualifier("local")
public class LocalCooperationFactory
    extends ScopedCooperationFactorySupport
{
  private final ConcurrentMap<String, CooperatingFuture<?>> localFutures = new ConcurrentHashMap<>();

  @Override
  @SuppressWarnings("unchecked")
  protected <T> CooperatingFuture<T> beginCooperation(final String scopedKey, final CooperatingFuture<T> future) {
    return (CooperatingFuture<T>) localFutures.putIfAbsent(scopedKey, future);
  }

  @Override
  protected <T> void endCooperation(final String scopedKey, final CooperatingFuture<T> future) {
    localFutures.remove(scopedKey, future);
  }

  @Override
  protected Stream<CooperatingFuture<?>> streamFutures(final String scope) {
    return localFutures.entrySet()
        .stream()
        .filter(entry -> entry.getKey().startsWith(scope))
        .map(Entry::getValue);
  }
}
