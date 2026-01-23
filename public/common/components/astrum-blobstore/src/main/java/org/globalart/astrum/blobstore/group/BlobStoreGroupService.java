
package org.globalart.astrum.blobstore.group;

import org.globalart.astrum.blobstore.api.BlobStore;

/**
 * Blob store group service.
 *
 * @since 3.15
 */
public interface BlobStoreGroupService
{
  /**
   * Are blob store groups enabled, i.e. can they be created and modified?
   */
  boolean isEnabled();

  /**
   * Takes a {@link BlobStore} and creates a {@link BlobStoreGroup} that contains the original blob store
   *
   * @param from a {@link BlobStore} to be converted into a group blob store
   * @param newNameForOriginal a new name for the original blob store
   * @return {@link BlobStoreGroup} that contains the original blob store
   */
  BlobStoreGroup convert(final BlobStore from, final String newNameForOriginal);
}
