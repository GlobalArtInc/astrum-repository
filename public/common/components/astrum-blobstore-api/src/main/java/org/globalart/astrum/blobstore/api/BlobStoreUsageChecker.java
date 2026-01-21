
package org.globalart.astrum.blobstore.api;

/**
 * Determines if a given {@link BlobId}, with a given {@code name}, is in use by a {@link BlobStore}.
 *
 * @since 3.5
 */
@FunctionalInterface
public interface BlobStoreUsageChecker
{
  boolean test(BlobStore blobStore, BlobId blobId, String blobName);
}
