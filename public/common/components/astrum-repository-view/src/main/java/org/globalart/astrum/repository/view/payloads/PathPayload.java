
package org.globalart.astrum.repository.view.payloads;

import java.io.BufferedInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.UncheckedIOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.view.Payload;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * A {@link Payload} backed by a {@link Path}
 */
public class PathPayload
    implements Payload
{
  private final Path path;

  private final String contentType;

  public PathPayload(final Path path, @Nullable final String contentType) {
    this.path = checkNotNull(path);
    this.contentType = contentType;
  }

  @Override
  public InputStream openInputStream() throws IOException {
    return new BufferedInputStream(Files.newInputStream(path, StandardOpenOption.READ));
  }

  @Override
  public long getSize() {
    try {
      return Files.size(path);
    }
    catch (IOException e) {
      throw new UncheckedIOException(e);
    }
  }

  @Override
  public String getContentType() {
    return contentType;
  }
}
