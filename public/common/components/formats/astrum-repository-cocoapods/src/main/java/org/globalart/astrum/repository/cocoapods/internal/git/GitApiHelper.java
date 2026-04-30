
package org.globalart.astrum.repository.cocoapods.internal.git;

import java.net.URI;

import javax.annotation.Nullable;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
public class GitApiHelper
{
  private static final String GITHUB_SNAPSHOT_URL_TEMPLATE = "%1$s/%2$s/%3$s/archive/%4$s.tar.gz";

  private static final String BITBUCKET_SNAPSHOT_URL_TEMPLATE = "%1$s/%2$s/%3$s/get/%4$s.tar.gz";

  private static final String GITLAB_SNAPSHOT_URL_TEMPLATE = "%1$s/%2$s/%3$s/-/archive/%4$s/%4$s.tar.gz";

  private static final String GITHUB_DEFAULT_REF = "HEAD";

  private static final String DEFAULT_BRANCH = "master";

  private final String githubUri;

  private final String bitbucketUri;

  private final String gitlabUri;

  @Autowired
  public GitApiHelper(
      @Value("${astrum.cocoapods.githubUri:https://github.com}") final String githubUri,
      @Value("${astrum.cocoapods.bitbucketUri:https://bitbucket.org}") final String bitbucketUri,
      @Value("${astrum.cocoapods.gitlabUri:https://gitlab.com}") final String gitlabUri)
  {
    this.githubUri = stripTrailingSlash(checkNotNull(githubUri));
    this.bitbucketUri = stripTrailingSlash(checkNotNull(bitbucketUri));
    this.gitlabUri = stripTrailingSlash(checkNotNull(gitlabUri));
  }

  public URI buildDownloadURI(final URI gitRepositoryUri, @Nullable final String ref) {
    GitArtifactInfo info = GitRepoUriParser.parseGitRepoUri(gitRepositoryUri, ref);

    GitProviderConfig config = prepareConfig(info.getHost());

    String downloadLink = String.format(
        config.template,
        config.baseUri,
        info.getVendor(),
        info.getRepository(),
        info.getRef() != null ? info.getRef() : config.defaultRef);

    return URI.create(downloadLink);
  }

  private GitProviderConfig prepareConfig(final String host) {
    switch (host) {
      case GitConstants.GITHUB_HOST:
        return new GitProviderConfig(GITHUB_SNAPSHOT_URL_TEMPLATE, githubUri, GITHUB_DEFAULT_REF);
      case GitConstants.BITBUCKET_HOST:
        return new GitProviderConfig(BITBUCKET_SNAPSHOT_URL_TEMPLATE, bitbucketUri, DEFAULT_BRANCH);
      case GitConstants.GITLAB_HOST:
        return new GitProviderConfig(GITLAB_SNAPSHOT_URL_TEMPLATE, gitlabUri, DEFAULT_BRANCH);
      default:
        throw new IllegalArgumentException("invalid host: " + host);
    }
  }

  private static String stripTrailingSlash(final String uri) {
    return uri.endsWith("/") ? uri.substring(0, uri.length() - 1) : uri;
  }

  private static class GitProviderConfig
  {
    private final String template;

    private final String baseUri;

    private final String defaultRef;

    GitProviderConfig(final String template, final String baseUri, final String defaultRef) {
      this.template = template;
      this.baseUri = baseUri;
      this.defaultRef = defaultRef;
    }
  }
}
