package org.globalart.astrum.repository.yum.internal.repodata;

import java.io.IOException;
import java.util.List;

public record Repodata(Repomd repomd, List<GeneratedMetadata> files, int packages, List<String> skipped)
    implements AutoCloseable
{
  @Override
  public void close() throws IOException {
    IOException failure = null;
    for (GeneratedMetadata file : files) {
      try {
        file.close();
      }
      catch (IOException e) {
        failure = e;
      }
    }
    if (failure != null) {
      throw failure;
    }
  }
}
