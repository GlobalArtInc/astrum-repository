
package org.globalart.astrum.repository.storage;

import java.util.Map;

import org.globalart.astrum.blobstore.api.BlobId;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.common.collect.NestedAttributesMap;

/**
 * Allows asset metadata to be stored in a blob store.
 *
 * @since 3.31
 */
public interface BlobMetadataStorage
{

  void attach(
      BlobStore blobStore,
      BlobId blobId,
      NestedAttributesMap componentAttributes,
      NestedAttributesMap assetAttributes,
      Map<String, String> checksums);
}
