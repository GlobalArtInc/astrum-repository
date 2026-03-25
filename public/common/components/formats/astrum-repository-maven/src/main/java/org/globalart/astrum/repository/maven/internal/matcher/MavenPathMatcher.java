
package org.globalart.astrum.repository.maven.internal.matcher;

import org.globalart.astrum.repository.maven.MavenPath;
import org.globalart.astrum.repository.maven.MavenPathParser;

/**
 * Matcher that matches any file, and sets {@link MavenPath} in context attributes.
 *
 * @since 3.0
 */
public class MavenPathMatcher
    extends MavenMatcherSupport
{
  public MavenPathMatcher(final MavenPathParser mavenPathParser) {
    super(mavenPathParser, (String path) -> Boolean.TRUE);
  }
}
