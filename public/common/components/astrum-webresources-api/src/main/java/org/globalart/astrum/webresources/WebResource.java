
package org.globalart.astrum.webresources;

import java.io.IOException;
import java.io.InputStream;
import java.net.URLConnection;

import javax.annotation.Nullable;

/**
 * A resource to be exposed via web (http/https) protocols.
 *
 * @since 2.8
 */
public interface WebResource
{
  long UNKNOWN_SIZE = -1L;

  long UNKNOWN_LAST_MODIFIED = 0L;

  String UNKNOWN_CONTENT_TYPE = "application/octet-stream";

  String HTML = "text/html";

  String PLAIN = "text/plain";

  String CSS = "text/css";

  String JAVASCRIPT = "application/x-javascript";

  /**
   * The path where the resource is mounted under the servlet-context.
   */
  String getPath();

  /**
   * The content-type of the resource, or {@code null} or {@link #UNKNOWN_CONTENT_TYPE} if unknown.
   */
  @Nullable
  String getContentType();

  /**
   * The size of the content, or {@link #UNKNOWN_SIZE} if unknown.
   *
   * @see URLConnection#getContentLengthLong()
   */
  long getSize();

  /**
   * The last modified time, or {@link #UNKNOWN_LAST_MODIFIED} if unknown.
   *
   * @see URLConnection#getLastModified()
   */
  long getLastModified();

  /**
   * True if the resource should be cached.
   */
  boolean isCacheable();

  /**
   * Resource content stream.
   */
  InputStream getInputStream() throws IOException;

  /**
   * Allows web-resources to prepare for handling requests.
   */
  interface Prepareable
  {
    WebResource prepare() throws IOException;
  }
}
