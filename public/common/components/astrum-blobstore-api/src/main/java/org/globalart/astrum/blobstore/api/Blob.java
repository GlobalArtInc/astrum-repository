
package org.globalart.astrum.blobstore.api;

import java.io.InputStream;
import java.net.URL;
import java.util.Map;
import java.util.Optional;

import javax.annotation.Nullable;

/**
 * A handle for binary data stored within a {@link BlobStore}.
 *
 * @since 3.0
 */
public interface Blob
{
  BlobId getId();

  /**
   * An immutable map of the headers that were provided when the blob was created.
   *
   * @throws BlobStoreException may be thrown if the blob is {@link BlobStore#delete deleted} or
   *           {@link BlobStore#delete hard deleted}.
   */
  Map<String, String> getHeaders();

  /**
   * Opens an input stream to the blob's content. The returned stream may be closed asynchronously if the blob is
   * {@link BlobStore#deleteHard(BlobId) hard deleted}. The input stream must support {@link InputStream#mark(int)}
   * because the ability to inspect the stream must be supported
   *
   * @throws BlobStoreException may be thrown if the blob is {@link BlobStore#delete deleted} or
   *           {@link BlobStore#delete hard deleted}.
   */
  InputStream getInputStream();

  /**
   * Provides metrics about this Blob.
   *
   * @throws BlobStoreException may be thrown if the blob is {@link BlobStore#delete deleted} or
   *           {@link BlobStore#delete hard deleted}.
   */
  BlobMetrics getMetrics();

  /**
   * Provides a URL for direct access (i.e. not through Astrum Repository) to the blob's content.
   */
  default Optional<URL> getRedirectUrl(final String action, final String name, @Nullable final String contentType) {
    return Optional.empty();
  }

  /**
   * @return if this payload would return a URL when calling getRedirectUrl
   */
  default boolean hasRedirect() {
    return false;
  }
}
