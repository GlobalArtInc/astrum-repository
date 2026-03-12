
package org.globalart.astrum.upgrade.datastore;

import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ConcurrentMap;

import org.globalart.astrum.common.cooperation2.Cooperation2;
import org.globalart.astrum.common.cooperation2.Cooperation2Factory;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Default {@link UpgradeCooperation} providing single-JVM cooperation via the injected
 * {@link Cooperation2Factory}. Overridden by {@code DistributedUpgradeCooperation} (via {@code @Primary})
 * when clustering is available.
 */
@Component
public class LocalUpgradeCooperation
    implements UpgradeCooperation
{
  private final Cooperation2Factory cooperationFactory;

  private final ConcurrentMap<String, Cooperation2> cooperations = new ConcurrentHashMap<>();

  @Autowired
  public LocalUpgradeCooperation(final Cooperation2Factory cooperationFactory) {
    this.cooperationFactory = checkNotNull(cooperationFactory);
  }

  @Override
  public Cooperation2 get(final String id) {
    // Memoize per id so repeated calls for the same scope share one Cooperation2 (and its coordination
    // state); the local Cooperation2 keeps its coordinating map per-instance, so building a fresh one per
    // call would defeat the "run only once" guarantee.
    return cooperations.computeIfAbsent(id, key -> cooperationFactory.configure().build(key));
  }
}
