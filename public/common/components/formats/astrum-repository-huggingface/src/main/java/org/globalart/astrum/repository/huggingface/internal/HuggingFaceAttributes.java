package org.globalart.astrum.repository.huggingface.internal;

import org.globalart.astrum.repository.huggingface.HuggingFaceFormat;

/**
 * Keys of the {@code huggingface} attributes of components and assets.
 */
public final class HuggingFaceAttributes
{
  public static final String ATTRIBUTES = HuggingFaceFormat.NAME;

  public static final String REPO_ID = "repoId";

  public static final String REPO_TYPE = "repoType";

  public static final String COMMIT = "commit";

  public static final String REVISIONS = "revisions";

  public static final String FILE = "file";

  public static final String ETAG = "etag";

  public static final String SIZE = "size";

  public static final String LFS = "lfs";

  private HuggingFaceAttributes() {
  }
}
