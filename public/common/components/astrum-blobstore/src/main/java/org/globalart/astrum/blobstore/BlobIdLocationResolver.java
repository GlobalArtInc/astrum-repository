
package org.globalart.astrum.blobstore;

import java.io.InputStream;
import java.util.Map;

import org.globalart.astrum.blobstore.api.BlobId;

/**
 * Interface providing operations to resolve {@link BlobId}s to locations and vice versa.
 *
 * @since 3.8
 */
public interface BlobIdLocationResolver
{
  /**
   * Resolves a {@link BlobId} to path-like {@link String}.
   */
  String getLocation(BlobId blobId);

  /**
   * Special use case to resolve a temporary location, even for non-temporary {@link BlobId}s.
   * Prefer {@link #getLocation(BlobId)}.
   */
  String getTemporaryLocation(BlobId id);

  /**
   * Safely constructs a {@link BlobId} from the {@link Map} argument to
   * {@link org.globalart.astrum.blobstore.api.BlobStore#create(InputStream, Map)}.
   */
  BlobId fromHeaders(Map<String, String> headers);
}
