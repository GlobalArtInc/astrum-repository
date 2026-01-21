
package org.globalart.astrum.blobstore.api;

/**
 * {@link BlobSession} supplier; for use by clients who don't need the full store API.
 *
 * @since 3.20
 */
public interface BlobSessionSupplier
{
  /**
   * Opens a new {@link BlobSession} against the named blob store.
   *
   * @throws BlobStoreNotFoundException if the store does not exist
   */
  BlobSession<?> openSession(String storeName);
}
