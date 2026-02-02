
package org.globalart.astrum.repository.view.payloads;

import java.io.IOException;
import java.io.InputStream;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.view.Payload;

import org.apache.http.Header;
import org.apache.http.HttpEntity;
import org.apache.http.HttpResponse;
import org.apache.http.util.EntityUtils;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Adapts {@link HttpEntity} to {@link Payload}.
 *
 * @since 3.0
 */
public class HttpEntityPayload
    implements Payload
{
  private final HttpResponse response;

  private final HttpEntity entity;

  public HttpEntityPayload(final HttpResponse response, final HttpEntity entity) {
    this.response = checkNotNull(response);
    this.entity = checkNotNull(entity);
  }

  @Override
  public InputStream openInputStream() throws IOException {
    return entity.getContent();
  }

  @Override
  public long getSize() {
    return entity.getContentLength();
  }

  @Nullable
  @Override
  public String getContentType() {
    Header header = entity.getContentType();
    if (header != null) {
      return header.getValue();
    }
    return null;
  }

  @Override
  public void close() throws IOException {
    EntityUtils.consume(entity);
  }
}
