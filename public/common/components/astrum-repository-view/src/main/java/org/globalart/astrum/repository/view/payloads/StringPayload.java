
package org.globalart.astrum.repository.view.payloads;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.Charset;
import java.nio.charset.StandardCharsets;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.view.Payload;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * String payload.
 *
 * @since 3.0
 */
public class StringPayload
    implements Payload
{
  private final byte[] contentBytes;

  private final Charset charset;

  private final String contentType;

  public StringPayload(final String content, final Charset charset, @Nullable final String contentType) {
    this.contentBytes = checkNotNull(content).getBytes(charset);
    this.charset = checkNotNull(charset);
    this.contentType = contentType;
  }

  public StringPayload(final String content, @Nullable final String contentType) {
    this(content, StandardCharsets.UTF_8, contentType);
  }

  public Charset getCharset() {
    return charset;
  }

  @Override
  public InputStream openInputStream() throws IOException {
    return new ByteArrayInputStream(contentBytes);
  }

  @Override
  public long getSize() {
    return contentBytes.length;
  }

  @Nullable
  @Override
  public String getContentType() {
    return contentType;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "size=" + contentBytes.length +
        ", charset=" + charset +
        ", contentType='" + contentType + '\'' +
        '}';
  }
}
