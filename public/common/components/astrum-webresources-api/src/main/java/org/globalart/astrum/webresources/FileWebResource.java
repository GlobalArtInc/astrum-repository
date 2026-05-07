
package org.globalart.astrum.webresources;

import java.io.BufferedInputStream;
import java.io.File;
import java.io.FileInputStream;
import java.io.IOException;
import java.io.InputStream;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * File-based {@link WebResource}.
 *
 * @since 2.8
 */
public class FileWebResource
    implements WebResource
{
  private final String path;

  private final String contentType;

  private final File file;

  private final boolean cachable;

  public FileWebResource(final File file, final String path, final String contentType, final boolean cachable) {
    this.file = checkNotNull(file);
    this.path = checkNotNull(path);
    this.contentType = checkNotNull(contentType);
    this.cachable = cachable;
  }

  @Override
  public boolean isCacheable() {
    return cachable;
  }

  @Override
  public String getPath() {
    return path;
  }

  @Override
  public String getContentType() {
    return contentType;
  }

  @Override
  public long getSize() {
    return file.length();
  }

  @Override
  public long getLastModified() {
    return file.lastModified();
  }

  @Override
  public InputStream getInputStream() throws IOException {
    return new BufferedInputStream(new FileInputStream(file));
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "path='" + path + '\'' +
        ", contentType='" + contentType + '\'' +
        ", file=" + file +
        ", cachable=" + cachable +
        '}';
  }
}
