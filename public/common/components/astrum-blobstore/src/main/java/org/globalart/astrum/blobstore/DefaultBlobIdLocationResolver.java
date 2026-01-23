
package org.globalart.astrum.blobstore;

import java.time.OffsetDateTime;
import java.util.Map;

import org.globalart.astrum.blobstore.api.BlobId;
import org.globalart.astrum.common.time.UTC;

import org.springframework.context.annotation.Primary;
import org.springframework.stereotype.Component;

import static java.util.UUID.randomUUID;
import static org.globalart.astrum.blobstore.api.BlobStore.BLOB_NAME_HEADER;
import static org.globalart.astrum.blobstore.api.BlobStore.DIRECT_PATH_BLOB_HEADER;
import static org.globalart.astrum.blobstore.api.BlobStore.TEMPORARY_BLOB_HEADER;

/**
 * Default {@link BlobIdLocationResolver}.
 *
 * @since 3.8
 */
@Primary
@Component
public class DefaultBlobIdLocationResolver
    implements BlobIdLocationResolver
{
  /**
   * Prefix indicating a temporary blob.
   */
  public static final String TEMPORARY_BLOB_ID_PREFIX = "tmp$";

  /**
   * Prefix indicating a direct-path blob.
   *
   * @see org.globalart.astrum.blobstore.api.BlobStore#DIRECT_PATH_BLOB_HEADER
   */
  public static final String DIRECT_PATH_BLOB_ID_PREFIX = "path$";

  protected final LocationStrategy volumeChapterLocationStrategy;

  protected final LocationStrategy temporaryLocationStrategy;

  protected final LocationStrategy directLocationStrategy;

  protected final LocationStrategy dateBasedLocationStrategy;

  public DefaultBlobIdLocationResolver() {
    this.volumeChapterLocationStrategy = new VolumeChapterLocationStrategy();
    this.temporaryLocationStrategy = new TemporaryLocationStrategy();
    this.directLocationStrategy = new DirectPathLocationStrategy();
    this.dateBasedLocationStrategy = new DateBasedLocationStrategy();
  }

  @Override
  public String getLocation(final BlobId id) {
    if (id.asUniqueString().startsWith(TEMPORARY_BLOB_ID_PREFIX)) {
      return temporaryLocationStrategy.location(id);
    }
    else if (id.asUniqueString().startsWith(DIRECT_PATH_BLOB_ID_PREFIX)) {
      return directLocationStrategy.location(id);
    }
    return getBlobIdLocation(id);
  }

  private String getBlobIdLocation(final BlobId blobId) {
    if (blobId.getBlobCreatedRef() != null) {
      return dateBasedLocationStrategy.location(blobId);
    }
    else {
      return volumeChapterLocationStrategy.location(blobId);
    }
  }

  @Override
  public String getTemporaryLocation(final BlobId id) {
    return temporaryLocationStrategy.location(id);
  }

  @Override
  public BlobId fromHeaders(final Map<String, String> headers) {
    OffsetDateTime blobCreatedRef = UTC.now();
    if (headers.containsKey(TEMPORARY_BLOB_HEADER)) {
      return new BlobId(TEMPORARY_BLOB_ID_PREFIX + randomUUID(), blobCreatedRef);
    }
    else if (headers.containsKey(DIRECT_PATH_BLOB_HEADER)) {
      return new BlobId(DIRECT_PATH_BLOB_ID_PREFIX + headers.get(BLOB_NAME_HEADER), blobCreatedRef);
    }
    return new BlobId(randomUUID().toString(), blobCreatedRef);
  }
}
