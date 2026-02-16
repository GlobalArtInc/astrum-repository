
package org.globalart.astrum.repository.httpbridge.internal;

import org.globalart.astrum.repository.BadRequestException;

import org.apache.commons.io.FilenameUtils;

/**
 * A utility class for parsing the repository name and remaining path out of a request URI.
 *
 * @since 3.0
 */
class RepositoryPath
{
  private final String repositoryName;

  private final String remainingPath;

  private RepositoryPath(final String repositoryName, final String remainingPath) {
    this.repositoryName = repositoryName;
    this.remainingPath = remainingPath;
  }

  public String getRepositoryName() {
    return repositoryName;
  }

  public String getRemainingPath() {
    return remainingPath;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "repositoryName='" + repositoryName + '\'' +
        ", remainingPath='" + remainingPath + '\'' +
        '}';
  }

  /**
   * Validate and parse the path.
   *
   * @throws BadRequestException if validation fails
   *
   * @return The parsed path
   */
  public static RepositoryPath parse(final String input) {
    String repo = validateAndExtractRepo(input);
    String path = validateAndExtractPath(input);
    return new RepositoryPath(repo, path);
  }

  private static String validateAndExtractRepo(final String input) {
    if (input == null || input.isEmpty()) {
      throw new BadRequestException("Repository path must not be null or empty");
    }

    if (!(input.charAt(0) == '/')) {
      throw new BadRequestException("Repository path must start with '/'");
    }

    int i = input.indexOf('/', 1);
    if (i == -1) {
      throw new BadRequestException("Repository path must have another '/' after initial '/'");
    }

    String repo = input.substring(1, i);
    if (".".equals(repo) || "..".equals(repo)) {
      throw new BadRequestException("Repository path must not contain a relative token");
    }

    return repo;
  }

  private static String validateAndExtractPath(final String input) {
    String path = input.substring(input.indexOf('/', 1), input.length());
    path = FilenameUtils.normalize(path, true); // unixSeparator:true is necessary to make this work on Windows.
    if (path == null) {
      throw new BadRequestException("Repository path must not contain a relative token");
    }
    return path;
  }
}
