
package org.globalart.astrum.repository.cocoapods.internal.git;

import java.net.URI;
import java.util.Arrays;

import javax.annotation.Nullable;

import static com.google.common.base.Preconditions.checkArgument;

public class GitRepoUriParser
{
  private static final String GIT_REPO_EXTENSION = ".git";

  private static final Integer VENDOR_POSITION = 1;

  private static final Integer REPO_POSITION = 2;

  private static final int GIT_PATH_SEGMENTS_COUNT = 3;

  private GitRepoUriParser() {}

  public static GitArtifactInfo parseGitRepoUri(final URI gitRepoUri, @Nullable final String ref) {
    checkArgument(isRepoSupported(gitRepoUri), "repository not supported: " + gitRepoUri);

    checkArgument(isGitUriFormatSupported(gitRepoUri), "invalid git path: " + gitRepoUri);

    String gitPath = gitRepoUri.getPath();
    if (gitPath.endsWith(GIT_REPO_EXTENSION)) {
      gitPath = gitPath.substring(0, gitPath.length() - GIT_REPO_EXTENSION.length());
    }

    String[] segments = gitPath.split("/");

    return new GitArtifactInfo(
        gitRepoUri.getHost(),
        segments[VENDOR_POSITION],
        extractRepository(segments, gitRepoUri.getHost()),
        ref);
  }

  private static String extractRepository(final String[] segments, final String host) {
    switch (host) {
      case GitConstants.GITHUB_HOST:
      case GitConstants.BITBUCKET_HOST:
        return segments[REPO_POSITION];
      case GitConstants.GITLAB_HOST:
        return String.join("/", Arrays.copyOfRange(segments, REPO_POSITION, segments.length));
      default:
        throw new IllegalStateException("invalid git repository: " + host);
    }
  }

  public static boolean isGitUriFormatSupported(final URI repoUri) {
    int segmentsCnt = repoUri.getPath().split("/").length;
    switch (repoUri.getHost()) {
      case GitConstants.GITHUB_HOST:
      case GitConstants.BITBUCKET_HOST:
        return segmentsCnt == GIT_PATH_SEGMENTS_COUNT;
      case GitConstants.GITLAB_HOST:
        return segmentsCnt >= GIT_PATH_SEGMENTS_COUNT;
      default:
        return false;
    }
  }

  public static boolean isRepoSupported(final URI repoUri) {
    return repoUri.getHost().equals(GitConstants.GITHUB_HOST)
        || repoUri.getHost().equals(GitConstants.BITBUCKET_HOST)
        || repoUri.getHost().equals(GitConstants.GITLAB_HOST);
  }
}
