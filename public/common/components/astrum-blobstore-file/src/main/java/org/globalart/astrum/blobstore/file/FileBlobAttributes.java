
package org.globalart.astrum.blobstore.file;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map;

import org.globalart.astrum.blobstore.BlobAttributesSupport;
import org.globalart.astrum.blobstore.api.BlobAttributes;
import org.globalart.astrum.blobstore.api.BlobMetrics;
import org.globalart.astrum.common.property.PropertiesFile;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * {@link BlobAttributes} backed by {@link PropertiesFile}
 *
 * @since 3.0
 */
public class FileBlobAttributes
    extends BlobAttributesSupport<PropertiesFile>
{
  public FileBlobAttributes(final Path path) {
    super(new PropertiesFile(path.toFile()), null, null);
  }

  public FileBlobAttributes(final Path path, final Map<String, String> headers, final BlobMetrics metrics) {
    super(new PropertiesFile(path.toFile()), checkNotNull(headers), checkNotNull(metrics));
  }

  /**
   * @since 3.2
   */
  public Path getPath() {
    return propertiesFile.getFile().toPath();
  }

  /**
   * Returns {@code false} if the attribute file is not found.
   */
  public boolean load() throws IOException {
    if (!Files.exists(getPath())) {
      return false;
    }
    propertiesFile.load();
    readFrom(propertiesFile);
    return true;
  }

  @Override
  public void store() throws IOException {
    writeTo(propertiesFile);
    propertiesFile.store();
  }

  @Override
  public void writeProperties() {
    writeTo(propertiesFile);
  }
}
