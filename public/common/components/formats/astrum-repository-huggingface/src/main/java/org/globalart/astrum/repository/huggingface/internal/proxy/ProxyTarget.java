package org.globalart.astrum.repository.huggingface.internal.proxy;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.huggingface.internal.HuggingFaceFile;
import org.globalart.astrum.repository.huggingface.internal.HuggingFacePaths;
import org.globalart.astrum.repository.huggingface.internal.RepoType;

/**
 * What a proxy request fetches: the information of a revision, or a file at a commit.
 */
record ProxyTarget(RepoType type, String repoId, String revision, @Nullable HuggingFaceFile file)
{
  static ProxyTarget revision(final RepoType type, final String repoId, final String revision) {
    return new ProxyTarget(type, repoId, revision, null);
  }

  static ProxyTarget file(final HuggingFaceFile file) {
    return new ProxyTarget(file.type(), file.repoId(), file.commit(), file);
  }

  String assetPath() {
    return file != null ? file.assetPath() : HuggingFacePaths.revisionPath(type, repoId, revision);
  }

  String remotePath() {
    return file != null
        ? HuggingFacePaths.fileUrl(type, repoId, file.commit(), file.path())
        : HuggingFacePaths.revisionUrl(type, repoId, revision);
  }

  /**
   * Content at a commit never changes, so it is never refreshed from the remote.
   */
  boolean immutable() {
    return file != null || HuggingFacePaths.isCommit(revision);
  }

  String describe() {
    return file != null ? file.path() + " of " + repoId + " at " + file.commit() : repoId + " at " + revision;
  }
}
