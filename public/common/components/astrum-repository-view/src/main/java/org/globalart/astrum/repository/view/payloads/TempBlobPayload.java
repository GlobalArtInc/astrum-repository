
package org.globalart.astrum.repository.view.payloads;

import java.io.IOException;
import java.io.InputStream;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.view.Payload;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.blobstore.api.BlobStore.CONTENT_TYPE_HEADER;

/**
 * PartPayload backed by a TempBlob. The storage layer will recognize this and unwrap it rather than creating another
 * {@link TempBlob}
 */
public class TempBlobPayload
    implements Payload
{
  private final String contentType;

  private final TempBlob tempBlob;

  public TempBlobPayload(final TempBlob tempBlob, @Nullable final String contentType) {
    this.tempBlob = checkNotNull(tempBlob);
    this.contentType = contentType;
  }

  public TempBlobPayload(final TempBlob tempBlob) {
    this.tempBlob = checkNotNull(tempBlob);
    this.contentType = tempBlob.getBlob().getHeaders().get(CONTENT_TYPE_HEADER);
  }

  @Override
  public InputStream openInputStream() throws IOException {
    return tempBlob.get();
  }

  @Override
  public long getSize() {
    return tempBlob.getBlob().getMetrics().getContentSize();
  }

  @Nullable
  @Override
  public String getContentType() {
    return contentType;
  }

  @SuppressWarnings("unchecked")
  public <T extends TempBlob> T getTempBlob() {
    return (T) tempBlob;
  }

  @Override
  public void close() throws IOException {
    tempBlob.close();
  }
}
