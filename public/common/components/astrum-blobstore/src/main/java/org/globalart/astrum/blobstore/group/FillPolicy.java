
package org.globalart.astrum.blobstore.group;

import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.blobstore.api.BlobStore;

/**
 * Chooses a group member to store a new blob.
 *
 * @since 3.14
 */
public interface FillPolicy
{

  String getName();

  /**
   * Choose the blob store group member to write a new blob to.
   */
  @Nullable
  BlobStore chooseBlobStore(BlobStoreGroup blobStoreGroup, Map<String, String> headers);
}
