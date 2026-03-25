
package org.globalart.astrum.repository.maven.internal.matcher;

import org.globalart.astrum.repository.maven.MavenPathParser;

/**
 * Matcher that matches anything in .meta folder (legacy paths from nx2)
 *
 * @since 3.3
 */
public class MavenNx2MetaFilesMatcher
    extends MavenMatcherSupport
{
  private static final String META_FILES_REQ_PATH = "/.meta/";

  public MavenNx2MetaFilesMatcher(final MavenPathParser mavenPathParser) {
    super(mavenPathParser, (String path) -> path != null && path.startsWith(META_FILES_REQ_PATH));
  }
}
