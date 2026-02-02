
package org.globalart.astrum.repository.view.payloads;

import java.io.IOException;
import java.io.InputStream;

import javax.annotation.Nullable;

import org.globalart.astrum.blobstore.api.Blob;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.repository.view.Payload;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.blobstore.api.BlobStore.CONTENT_TYPE_HEADER;

/**
 * A {@link Payload} which is backed by a {@link Blob} which is not attached to any assets. Primarily intended for use
 * in reconcile.
 */
public class DetachedBlobPayload
    implements Payload
{
  private final Blob blob;

  @Nullable
  private final BlobStore sourceBlobStore;

  private final String contentType;

  public DetachedBlobPayload(final Blob blob) {
    this.blob = checkNotNull(blob);
    this.sourceBlobStore = null;
    this.contentType = blob.getHeaders().get(CONTENT_TYPE_HEADER);
  }

  public DetachedBlobPayload(final Blob blob, final BlobStore sourceBlobStore) {
    this.blob = checkNotNull(blob);
    this.sourceBlobStore = checkNotNull(sourceBlobStore);
    this.contentType = blob.getHeaders().get(CONTENT_TYPE_HEADER);
  }

  public Blob getBlob() {
    return blob;
  }

  @Nullable
  public BlobStore getSourceBlobStore() {
    return sourceBlobStore;
  }

  @Override
  public InputStream openInputStream() throws IOException {
    return blob.getInputStream();
  }

  @Override
  public long getSize() {
    return blob.getMetrics().getContentSize();
  }

  @Nullable
  @Override
  public String getContentType() {
    return contentType;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "blob=" + blob +
        ", contentType='" + contentType + '\'' +
        '}';
  }
}
