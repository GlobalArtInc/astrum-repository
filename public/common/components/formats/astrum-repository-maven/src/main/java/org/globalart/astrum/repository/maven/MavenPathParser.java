
package org.globalart.astrum.repository.maven;

import javax.annotation.Nonnull;

/**
 * Component representing Maven layout specific bits, like parsing request paths into {@link MavenPath}.
 *
 * @since 3.0
 */
public interface MavenPathParser
{
  /**
   * Parses path into {@link MavenPath}.
   */
  @Nonnull
  MavenPath parsePath(String path);

  /**
   * Parses path into {@link MavenPath} with optional case sensitivity
   *
   * @since 3.7
   */
  @Nonnull
  MavenPath parsePath(String path, boolean caseSensitive);

  /**
   * Returns {@code true} if passed in path represent repository metadata path.
   */
  boolean isRepositoryMetadata(MavenPath path);

  /**
   * Returns {@code true} if passed in path represents a repository index file path.
   */
  boolean isRepositoryIndex(MavenPath path);
}
