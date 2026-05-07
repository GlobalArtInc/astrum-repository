
package org.globalart.astrum.webresources;

import java.io.IOException;
import java.io.InputStream;
import java.net.URL;
import java.net.URLConnection;

import com.google.common.base.Strings;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * URL-based {@link WebResource} implementation.
 *
 * @since 2.8
 */
public class UrlWebResource
    implements WebResource
{
  private final URL url;

  private final String path;

  private final boolean cacheable;

  private final String contentType;

  private final long size;

  private final long lastModified;

  public UrlWebResource(final URL url, final String path, final String contentType) {
    this(url, path, contentType, true);
  }

  public UrlWebResource(final URL url, final String path, final String contentType, final boolean cacheable) {
    this.url = checkNotNull(url);
    this.path = checkNotNull(path);
    this.cacheable = cacheable;

    // open connection to get details about the resource
    try {
      final URLConnection connection = this.url.openConnection();
      try (final InputStream ignore = connection.getInputStream()) {
        if (Strings.isNullOrEmpty(contentType)) {
          this.contentType = connection.getContentType();
        }
        else {
          this.contentType = contentType;
        }

        // support for legacy int and modern long content-length
        long detectedSize = connection.getContentLengthLong();
        if (detectedSize == -1) {
          detectedSize = connection.getContentLength();
        }
        this.size = detectedSize;

        this.lastModified = connection.getLastModified();
      }
    }
    catch (IOException e) {
      throw new IllegalArgumentException("Resource inaccessible: " + url, e);
    }
  }

  @Override
  public String getPath() {
    if (path != null) {
      return path;
    }
    else {
      return url.getPath();
    }
  }

  @Override
  public long getSize() {
    return size;
  }

  @Override
  public String getContentType() {
    return contentType;
  }

  @Override
  public long getLastModified() {
    return lastModified;
  }

  @Override
  public InputStream getInputStream() throws IOException {
    return url.openStream();
  }

  @Override
  public boolean isCacheable() {
    return cacheable;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "url=" + url +
        ", path='" + path + '\'' +
        ", cacheable=" + cacheable +
        ", contentType='" + contentType + '\'' +
        ", size=" + size +
        ", lastModified=" + lastModified +
        '}';
  }
}
