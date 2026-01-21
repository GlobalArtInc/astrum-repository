
package org.globalart.astrum.blobstore.api;

/**
 * Provides information about active blob store migrations.
 *
 * This interface allows blob stores to determine if a migration is currently in progress,
 * enabling optimizations like conditional existence checking only during migration windows.
 *
 * @since 3.next
 */
public interface BlobStoreMigrationStateProvider
{
  /**
   * Check if any blob store migration is currently active.
   *
   * @return true if at least one blob store migration is in progress, false otherwise
   */
  boolean isMigrationActive();

  /**
   * Check if a specific blob store is currently being migrated from or to.
   *
   * @param blobStoreName the name of the blob store to check
   * @return true if the specified blob store is involved in an active migration, false otherwise
   */
  boolean isBlobStoreMigrating(String blobStoreName);
}
