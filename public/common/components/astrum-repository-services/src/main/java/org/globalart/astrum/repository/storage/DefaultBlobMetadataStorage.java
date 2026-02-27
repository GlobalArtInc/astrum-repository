
package org.globalart.astrum.repository.storage;

import java.util.Map;

import org.globalart.astrum.blobstore.api.BlobId;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.common.collect.NestedAttributesMap;

import org.springframework.context.annotation.Primary;
import org.springframework.stereotype.Component;

/**
 * Default implementation of {@link BlobMetadataStorage} that doesn't store metadata in the blob store.
 *
 * @since 3.31
 */
@Primary
@Component
public class DefaultBlobMetadataStorage
    implements BlobMetadataStorage
{
  @Override
  public void attach(
      final BlobStore blobStore,
      final BlobId blobId,
      final NestedAttributesMap componentAttributes,
      final NestedAttributesMap assetAttributes,
      final Map<String, String> checksums)
  {
    // no-op
  }
}
