
package org.globalart.astrum.upgrade.datastore;

import org.globalart.astrum.common.cooperation2.Cooperation2;

/**
 * Provides a {@link Cooperation2} instance for use by UPGRADE-phase migration steps.
 *
 * <p>
 * The default {@link LocalUpgradeCooperation} provides single-JVM cooperation. When clustering is
 * available, {@code DistributedUpgradeCooperation} (in {@code astrum-datastore-clustering}) overrides it
 * (via {@code @Primary}) so that cooperation is coordinated cluster-wide — important for migrations that
 * must run their work only once across the cluster.
 * </p>
 */
public interface UpgradeCooperation
{
  /**
   * Returns the {@link Cooperation2} for the given cooperation id/scope.
   *
   * <p>
   * Implementations memoize by id: repeated calls with the same id return the same {@link Cooperation2}
   * instance (and share its coordination state), so the "run only once" guarantee holds even if a caller
   * does not cache the result.
   * </p>
   */
  Cooperation2 get(String id);
}
