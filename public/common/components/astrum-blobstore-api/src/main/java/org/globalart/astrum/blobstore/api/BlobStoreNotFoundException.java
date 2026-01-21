
package org.globalart.astrum.blobstore.api;

/**
 * Thrown when a named {@link BlobStore} was not found.
 *
 * @since 3.20
 */
public class BlobStoreNotFoundException
    extends RuntimeException
{
  private static final long serialVersionUID = 3153805672148771502L;

  public BlobStoreNotFoundException(final String storeName) {
    super("Blob store not found: " + storeName);
  }
}
