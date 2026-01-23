
package org.globalart.astrum.blobstore;

import org.globalart.astrum.blobstore.api.BlobId;

/**
 * Strategy determines the location of a blob file in a store.
 *
 * Implementations might use algorithms to divide files up into multiple directories for easier management.
 *
 * @since 3.0
 */
public interface LocationStrategy
{
  /**
   * Returns the location where blob file should exist.
   */
  String location(BlobId blobId);
}
