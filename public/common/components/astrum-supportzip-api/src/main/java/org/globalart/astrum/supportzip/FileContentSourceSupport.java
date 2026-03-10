
package org.globalart.astrum.supportzip;

import java.io.BufferedInputStream;
import java.io.File;
import java.io.FileInputStream;
import java.io.InputStream;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Preconditions.checkState;

/**
 * Support for including existing files as {@link SupportBundle.ContentSource}.
 *
 * @since 2.7
 */
public class FileContentSourceSupport
    extends ContentSourceSupport
{
  protected final File file;

  /**
   * @since 3.0
   */
  public FileContentSourceSupport(final Type type, final String path, final File file, final Priority priority) {
    super(type, path, priority);
    this.file = checkNotNull(file);
  }

  public FileContentSourceSupport(final Type type, final String path, final File file) {
    this(type, path, file, Priority.DEFAULT);
  }

  @Override
  public void prepare() throws Exception {
    checkState(file.exists());
  }

  @Override
  public long getSize() {
    checkState(file.exists());
    return file.length();
  }

  @Override
  public InputStream getContent() throws Exception {
    checkState(file.exists());
    log.debug("Reading: {}", file);
    return new BufferedInputStream(new FileInputStream(file));
  }

  @Override
  public void cleanup() throws Exception {
    // nothing
  }
}
