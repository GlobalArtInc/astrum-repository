
package org.globalart.astrum.repository.content.fluent;

import java.io.InputStream;
import java.nio.file.Path;
import java.util.Map;
import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.blobstore.api.Blob;
import org.globalart.astrum.blobstore.api.BlobRef;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreMetrics;
import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import com.google.common.hash.HashCode;

/**
 * Fluent API for ingesting blobs.
 *
 * @since 3.21
 */
public interface FluentBlobs
{
  /**
   * Ingests the given stream as a temporary blob with the requested hashing.
   */
  TempBlob ingest(InputStream in, @Nullable String contentType, Iterable<HashAlgorithm> hashing);

  /**
   * Ingests the given stream as a temporary blob with custom headers and the requested hashing.
   */
  TempBlob ingest(
      InputStream in,
      @Nullable String contentType,
      Map<String, String> headers,
      Iterable<HashAlgorithm> hashing);

  /**
   * Ingests the given payload as a temporary blob with the requested hashing.
   */
  default TempBlob ingest(final Payload payload, final Iterable<HashAlgorithm> hashing) {
    return ingest(payload, Map.of(), hashing);
  }

  /**
   * Ingests the given payload as a temporary blob with the requested hashing.
   */
  TempBlob ingest(Payload payload, Map<String, String> headers, Iterable<HashAlgorithm> hashing);

  TempBlob ingest(final Blob srcBlob, final BlobStore srcStore, final Map<HashAlgorithm, HashCode> hashes);

  /**
   * Ingests the given payload as a temporary blob with the requested hashing.
   *
   * @param path the path to the file on the local file system
   * @param contentType the content type of the provided file if known
   * @param hashing the hashing algorithms to use for the blob
   * @param requireHardLink when true ingest will fail if the attempt to hard link fails, otherwise an attempt will be
   *          made to copy the file content.
   *
   * @since 3.41
   */
  TempBlob ingest(Path path, @Nullable String contentType, Iterable<HashAlgorithm> hashing, boolean requireHardLink);

  /**
   * Ingests a blob from a {@code sourceFile} via hard-linking.
   *
   * @since 3.29
   */
  Blob ingest(Path sourceFile, Map<String, String> headers, HashCode sha1, long size);

  /**
   * Fetches the Blob associated with the specified BlobRef or Optional.empty() if
   * no such Blob exists.
   *
   * @since 3.29
   */
  Optional<Blob> blob(BlobRef blobRef);

  /**
   * @since 3.29
   */
  BlobStoreMetrics getMetrics();
}
