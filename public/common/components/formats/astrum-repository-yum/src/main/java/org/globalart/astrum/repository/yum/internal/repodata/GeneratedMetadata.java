package org.globalart.astrum.repository.yum.internal.repodata;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

public record GeneratedMetadata(MetadataType type, Path file, RepomdData data)
    implements AutoCloseable
{
  public String fileName() {
    return data.location().substring(data.location().lastIndexOf('/') + 1);
  }

  @Override
  public void close() throws IOException {
    Files.deleteIfExists(file);
  }
}
