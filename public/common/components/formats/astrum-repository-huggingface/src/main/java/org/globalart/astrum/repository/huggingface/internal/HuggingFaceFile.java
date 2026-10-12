package org.globalart.astrum.repository.huggingface.internal;

import java.util.Set;
import java.util.TreeSet;

import javax.annotation.Nullable;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * A file of a repository at a commit, with what the Hub told about it before it was fetched.
 *
 * @param revisions the branches and tags known to point at the commit
 * @param etag the ETag the Hub answers for the file, when known
 * @param lfsSha256 the SHA-256 the content must have, for LFS files
 * @param size the size the content must have, or -1
 */
public record HuggingFaceFile(RepoType type, String repoId, String commit, String path, Set<String> revisions,
                              @Nullable String etag, @Nullable String lfsSha256, long size)
{
  public HuggingFaceFile {
    checkNotNull(type);
    checkNotNull(repoId);
    checkNotNull(commit);
    checkNotNull(path);
    revisions = Set.copyOf(revisions);
  }

  public static HuggingFaceFile of(final RepoType type, final String repoId, final String commit, final String path,
                                   final Set<String> revisions, @Nullable final RepoFile listed)
  {
    if (listed == null) {
      return new HuggingFaceFile(type, repoId, commit, path, revisions, null, null, -1);
    }
    return new HuggingFaceFile(type, repoId, commit, path, revisions, listed.etag(), listed.lfsSha256(),
        listed.contentSize());
  }

  public String assetPath() {
    return HuggingFacePaths.filePath(type, repoId, commit, path);
  }

  public HuggingFaceFile withRemote(final Set<String> moreRevisions, @Nullable final String remoteEtag,
                                    @Nullable final String remoteSha256, final long remoteSize)
  {
    Set<String> all = new TreeSet<>(revisions);
    all.addAll(moreRevisions);
    return new HuggingFaceFile(type, repoId, commit, path, all, etag != null ? etag : remoteEtag,
        lfsSha256 != null ? lfsSha256 : remoteSha256, size >= 0 ? size : remoteSize);
  }
}
