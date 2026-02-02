
package org.globalart.astrum.repository.view.payloads;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.view.Payload;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Bytes payload.
 *
 * @since 3.0
 */
public class BytesPayload
    implements Payload
{
  private final byte[] content;

  private final String contentType;

  public BytesPayload(final byte[] content, @Nullable final String contentType) {
    this.content = checkNotNull(content);
    this.contentType = contentType;
  }

  @Override
  public InputStream openInputStream() throws IOException {
    return new ByteArrayInputStream(content);
  }

  @Override
  public long getSize() {
    return content.length;
  }

  @Nullable
  @Override
  public String getContentType() {
    return contentType;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "size=" + getSize() +
        ", contentType='" + contentType + '\'' +
        '}';
  }
}
