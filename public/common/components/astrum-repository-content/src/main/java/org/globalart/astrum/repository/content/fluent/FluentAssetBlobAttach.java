
package org.globalart.astrum.repository.content.fluent;

import java.util.Map;

import org.globalart.astrum.blobstore.api.Blob;
import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import com.google.common.hash.HashCode;

/**
 * Encapsulates methods for updating a {@link FluentAsset} with a new blob.
 *
 * @since 3.30
 */
public interface FluentAssetBlobAttach
{
  /**
   * Converts a temporary blob into a permanent blob and attaches it to this asset.
   */
  FluentAsset attach(TempBlob blob);

  /**
   * Attaches an existing blob to this asset.
   */
  FluentAsset attach(Blob blob, Map<HashAlgorithm, HashCode> checksums);

  /**
   * Attaches an existing blob to this asset ignoring the repository's write policy.
   *
   * @since 3.31
   */
  FluentAsset attachIgnoringWritePolicy(Blob blob, Map<HashAlgorithm, HashCode> checksums);
}
