package org.globalart.astrum.repository.huggingface.internal;

import java.io.ByteArrayInputStream;
import java.io.InputStream;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.view.Payload;

/**
 * The payload of a {@code HEAD} answer for a file that is not cached yet: its size, no body.
 */
public class HeadPayload
    implements Payload
{
  private final long size;

  @Nullable
  private final String contentType;

  public HeadPayload(final long size, @Nullable final String contentType) {
    this.size = size;
    this.contentType = contentType;
  }

  @Override
  public InputStream openInputStream() {
    return new ByteArrayInputStream(new byte[0]);
  }

  @Override
  public long getSize() {
    return size;
  }

  @Nullable
  @Override
  public String getContentType() {
    return contentType;
  }
}
