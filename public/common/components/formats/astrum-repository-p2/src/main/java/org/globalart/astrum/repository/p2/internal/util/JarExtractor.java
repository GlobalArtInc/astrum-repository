
package org.globalart.astrum.repository.p2.internal.util;

import java.io.IOException;
import java.util.Optional;
import java.util.jar.JarEntry;
import java.util.jar.JarInputStream;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.p2.internal.exception.AttributeParsingException;
import org.globalart.astrum.repository.view.payloads.TempBlob;

public abstract class JarExtractor<T>
{
  private final TempBlobConverter tempBlobConverter;

  protected JarExtractor(final TempBlobConverter tempBlobConverter) {
    this.tempBlobConverter = tempBlobConverter;
  }

  protected Optional<T> getSpecificEntity(
      final TempBlob tempBlob,
      final String extension,
      @Nullable final String startNameForSearch) throws IOException, AttributeParsingException
  {
    try (JarInputStream jis = getJarStreamFromBlob(tempBlob, extension)) {
      Optional<T> manifestEntity = fromManifest(jis);
      if (manifestEntity.isPresent()) {
        return manifestEntity;
      }
      JarEntry jarEntry;
      while ((jarEntry = jis.getNextJarEntry()) != null) {
        if (startNameForSearch != null && jarEntry.getName().startsWith(startNameForSearch)) {
          return Optional.ofNullable(createSpecificEntity(jis, jarEntry));
        }
      }
    }

    return Optional.empty();
  }

  protected Optional<T> fromManifest(final JarInputStream jis) {
    return Optional.empty();
  }

  protected abstract T createSpecificEntity(final JarInputStream jis, final JarEntry jarEntry) throws IOException, AttributeParsingException;

  private JarInputStream getJarStreamFromBlob(final TempBlob tempBlob, final String extension) throws IOException {
    if (extension != null && extension.endsWith("pack.gz")) {
      return new JarInputStream(tempBlobConverter.getJarFromPackGz(tempBlob));
    }
    return new JarInputStream(tempBlob.get());
  }
}
