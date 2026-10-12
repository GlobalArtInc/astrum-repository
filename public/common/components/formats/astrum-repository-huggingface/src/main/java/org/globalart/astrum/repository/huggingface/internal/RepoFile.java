package org.globalart.astrum.repository.huggingface.internal;

import javax.annotation.Nullable;

/**
 * A file of a repository revision as the Hub lists it in the siblings of the revision information.
 *
 * @param blobId the git object id of the file, of the pointer file for LFS files
 * @param lfsSha256 the SHA-256 of the content for LFS files
 */
public record RepoFile(String path, @Nullable String blobId, long size, @Nullable String lfsSha256, long lfsSize,
                       long pointerSize)
{
  public boolean lfs() {
    return lfsSha256 != null;
  }

  /**
   * The ETag the Hub answers for the file: the SHA-256 for LFS files, the git object id otherwise.
   */
  @Nullable
  public String etag() {
    return lfs() ? lfsSha256 : blobId;
  }

  public long contentSize() {
    return lfs() ? lfsSize : size;
  }
}
