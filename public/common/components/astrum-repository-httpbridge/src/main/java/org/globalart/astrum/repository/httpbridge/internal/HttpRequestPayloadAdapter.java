
package org.globalart.astrum.repository.httpbridge.internal;

import java.io.IOException;
import java.io.InputStream;

import javax.annotation.Nullable;
import jakarta.servlet.http.HttpServletRequest;

import org.globalart.astrum.repository.view.Payload;

/**
 * HTTP request payload adapts {@link HttpServletRequest} body-content to {@link Payload}.
 *
 * @since 3.0
 */
class HttpRequestPayloadAdapter
    implements Payload
{
  private final HttpServletRequest request;

  private final String contentType;

  private final long size;

  public HttpRequestPayloadAdapter(final HttpServletRequest request) {
    this.request = request;
    this.contentType = request.getContentType();
    this.size = request.getContentLength();
  }

  @Nullable
  @Override
  public String getContentType() {
    return contentType;
  }

  @Override
  public long getSize() {
    return size;
  }

  @Override
  public InputStream openInputStream() throws IOException {
    return request.getInputStream();
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "contentType='" + contentType + '\'' +
        ", size=" + size +
        '}';
  }
}
