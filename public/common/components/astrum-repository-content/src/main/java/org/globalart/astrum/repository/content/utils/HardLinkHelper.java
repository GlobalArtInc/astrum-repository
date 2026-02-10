
package org.globalart.astrum.repository.content.utils;

import java.io.BufferedInputStream;
import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HashMap;
import java.util.Map;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.blobstore.api.Blob;
import org.globalart.astrum.mime.MimeSupport;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentBlobs;

import com.google.common.hash.HashCode;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.blobstore.api.BlobStore.BLOB_NAME_HEADER;
import static org.globalart.astrum.blobstore.api.BlobStore.CONTENT_TYPE_HEADER;
import org.springframework.stereotype.Component;

/**
 * @since 3.35
 * @deprecated please use the appropriate ingest methods on {@code StorageFacet} and {@code FluentBlobs}
 */
@Deprecated
@Component
public class HardLinkHelper
{
  private final MimeSupport mimeSupport;

  @Autowired
  public HardLinkHelper(final MimeSupport mimeSupport) {
    this.mimeSupport = checkNotNull(mimeSupport);
  }

  /**
   * Ingests a blob from a {@code content} via hard-linking.
   */
  public Blob ingestHardLink(
      final Repository repository,
      final File content,
      final HashCode hashCodeSha1) throws IOException
  {
    checkNotNull(repository);
    checkNotNull(content);
    checkNotNull(hashCodeSha1);

    Map<String, String> headers = new HashMap<>();
    String path = content.getPath();
    headers.put(BLOB_NAME_HEADER, path);
    String contentType = detectMimeType(content);
    headers.put(CONTENT_TYPE_HEADER, contentType);

    Path contentPath = content.toPath();

    FluentBlobs fluentBlobs = repository.facet(ContentFacet.class).blobs();

    return fluentBlobs.ingest(contentPath, headers, hashCodeSha1, Files.size(contentPath));
  }

  /**
   * Ingests a blob from a {@code content} via hard-linking.
   */
  public Blob ingestHardLink(
      final Repository repository,
      final File content,
      final HashCode hashCodeSha1,
      final String contentType) throws IOException
  {
    checkNotNull(repository);
    checkNotNull(content);
    checkNotNull(hashCodeSha1);
    checkNotNull(contentType);

    Map<String, String> headers = new HashMap<>();
    String path = content.getPath();
    headers.put(BLOB_NAME_HEADER, path);
    headers.put(CONTENT_TYPE_HEADER, contentType);

    Path contentPath = content.toPath();

    FluentBlobs fluentBlobs = repository.facet(ContentFacet.class).blobs();

    return fluentBlobs.ingest(contentPath, headers, hashCodeSha1, Files.size(contentPath));
  }

  private String detectMimeType(final File content) throws IOException {
    String path = content.getPath();
    Path contentPath = content.toPath();
    try (InputStream inputStream = new BufferedInputStream(Files.newInputStream(contentPath))) {
      return mimeSupport.detectMimeType(inputStream, path);
    }
  }
}
