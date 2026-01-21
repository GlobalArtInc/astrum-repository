
package org.globalart.astrum.blobstore.api;

/**
 * Indicates that an error occurred connecting to the source of the blobstore.
 *
 * @since 3.31
 */
public class BlobStoreConnectionException
    extends RuntimeException
{
  public BlobStoreConnectionException(final String message) {
    super(message);
  }
}
