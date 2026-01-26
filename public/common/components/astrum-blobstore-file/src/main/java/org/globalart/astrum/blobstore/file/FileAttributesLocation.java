
package org.globalart.astrum.blobstore.file;

import java.nio.file.Path;

import org.globalart.astrum.blobstore.AttributesLocation;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Location for FileBlobStore attributes files
 *
 * @since 3.15
 */
public class FileAttributesLocation
    implements AttributesLocation
{
  private final Path path;

  public FileAttributesLocation(final Path path) {
    this.path = checkNotNull(path);
  }

  @Override
  public String getFileName() {
    return path.toFile().getName();
  }

  @Override
  public String getFullPath() {
    return path.toString();
  }

  public Path getPath() {
    return path;
  }
}
