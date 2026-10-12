package org.globalart.astrum.repository.huggingface.internal;

import javax.annotation.Nullable;

import static com.google.common.base.Preconditions.checkNotNull;

public record HuggingFaceRequest(Kind kind, @Nullable RepoType type, @Nullable String repoId, @Nullable String revision,
                                 String path)
{
  public enum Kind
  {
    INFO,
    TREE,
    PATHS_INFO,
    FILE,
    STORED
  }

  public HuggingFaceRequest {
    checkNotNull(kind);
    checkNotNull(path);
  }

  public static HuggingFaceRequest stored(final String path) {
    return new HuggingFaceRequest(Kind.STORED, null, null, null, path);
  }
}
