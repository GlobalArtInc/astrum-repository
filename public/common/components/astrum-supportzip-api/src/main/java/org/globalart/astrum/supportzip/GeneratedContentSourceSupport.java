
package org.globalart.astrum.supportzip;

import java.io.BufferedInputStream;
import java.io.File;
import java.io.FileInputStream;
import java.io.InputStream;
import java.nio.file.Files;

import static com.google.common.base.Preconditions.checkState;

/**
 * Support for generated {@link SupportBundle.ContentSource} implementations.
 *
 * These sources will buffer output to a file on prepare.
 *
 * @since 2.7
 */
public abstract class GeneratedContentSourceSupport
    extends ContentSourceSupport
{
  protected File file;

  public GeneratedContentSourceSupport(final Type type, final String path) {
    super(type, path);
  }

  /**
   * @since 3.0
   */
  public GeneratedContentSourceSupport(final Type type, final String path, final Priority priority) {
    super(type, path, priority);
  }

  @Override
  public void prepare() throws Exception {
    checkState(file == null);
    file = File.createTempFile(getPath().replaceAll("/", "-") + "-", ".tmp").getCanonicalFile();
    log.trace("Preparing: {}", file);
    generate(file);
  }

  protected abstract void generate(File file) throws Exception;

  @Override
  public long getSize() {
    checkState(file.exists());
    return file.length();
  }

  @Override
  public InputStream getContent() throws Exception {
    checkState(file.exists());
    return new BufferedInputStream(new FileInputStream(file));
  }

  @Override
  public void cleanup() throws Exception {
    if (file != null) {
      log.trace("Cleaning: {}", file);
      Files.delete(file.toPath());
      file = null;
    }
  }
}
