
package org.globalart.astrum.repository.maven.internal.matcher;

import org.globalart.astrum.repository.maven.MavenPathParser;

/**
 * Matcher that matches anything in .meta folder (legacy paths)
 *
 * @since 3.3
 */
public class MavenLegacyMetaFilesMatcher
    extends MavenMatcherSupport
{
  private static final String META_FILES_REQ_PATH = "/.meta/";

  public MavenLegacyMetaFilesMatcher(final MavenPathParser mavenPathParser) {
    super(mavenPathParser, (String path) -> path != null && path.startsWith(META_FILES_REQ_PATH));
  }
}
