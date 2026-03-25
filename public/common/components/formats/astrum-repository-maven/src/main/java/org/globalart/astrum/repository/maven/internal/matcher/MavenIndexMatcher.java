
package org.globalart.astrum.repository.maven.internal.matcher;

import org.globalart.astrum.repository.maven.MavenPath;
import org.globalart.astrum.repository.maven.MavenPathParser;

import static org.globalart.astrum.repository.maven.internal.Constants.INDEX_FILE_BASE_PATH;

/**
 * Matcher that matches for published Maven index only, and sets {@link MavenPath} in context attributes.
 *
 * @since 3.0
 */
public class MavenIndexMatcher
    extends MavenMatcherSupport
{
  public MavenIndexMatcher(final MavenPathParser mavenPathParser) {
    super(mavenPathParser, (String path) -> path != null && path.startsWith("/" + INDEX_FILE_BASE_PATH));
  }
}
