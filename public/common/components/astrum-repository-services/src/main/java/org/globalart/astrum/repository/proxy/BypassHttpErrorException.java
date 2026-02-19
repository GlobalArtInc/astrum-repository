
package org.globalart.astrum.repository.proxy;

import com.google.common.collect.ListMultimap;

/**
 * @since 3.18.1
 */
public class BypassHttpErrorException
    extends RuntimeException
{
  private final int httpCode;

  private final String reason;

  private final ListMultimap<String, String> headers;

  private final String body;

  private final String contentType;

  public BypassHttpErrorException(final int httpCode, final String reason, final ListMultimap<String, String> headers) {
    this.httpCode = httpCode;
    this.reason = reason;
    this.headers = headers;
    this.body = "";
    this.contentType = "";
  }

  public BypassHttpErrorException(
      final int httpCode,
      final String reason,
      final ListMultimap<String, String> headers,
      final String body,
      final String contentType)
  {
    this.httpCode = httpCode;
    this.reason = reason;
    this.headers = headers;
    this.body = body;
    this.contentType = contentType;
  }

  public int getStatusCode() {
    return httpCode;
  }

  public String getReason() {
    return reason;
  }

  public ListMultimap<String, String> getHeaders() {
    return headers;
  }

  public String getBody() {
    return body;
  }

  public String getContentType() {
    return contentType;
  }
}
