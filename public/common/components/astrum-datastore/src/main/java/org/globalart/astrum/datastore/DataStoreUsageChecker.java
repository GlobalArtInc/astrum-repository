
package org.globalart.astrum.datastore;

/**
 * Checks whether data stores are currently being used by the system.
 *
 * @since 3.19
 */
public interface DataStoreUsageChecker
{
  /**
   * @return {@code true} if the named data store is in use.
   */
  boolean isDataStoreUsed(String storeName);
}
