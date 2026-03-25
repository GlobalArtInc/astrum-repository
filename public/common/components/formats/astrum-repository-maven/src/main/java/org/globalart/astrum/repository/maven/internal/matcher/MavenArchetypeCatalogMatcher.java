
package org.globalart.astrum.repository.maven.internal.matcher;

import org.globalart.astrum.repository.maven.MavenPathParser;
import org.globalart.astrum.repository.maven.internal.Constants;

/**
 * Matcher that matches for Maven archetype catalog only.
 *
 * @since 3.0
 */
public class MavenArchetypeCatalogMatcher
    extends MavenMatcherSupport
{
  private static final String ARCHETYPE_CATALOG_REQ_PATH = "/" + Constants.ARCHETYPE_CATALOG_FILENAME;

  public MavenArchetypeCatalogMatcher(final MavenPathParser mavenPathParser) {
    super(mavenPathParser,
        withHashes((String path) -> path.equals(ARCHETYPE_CATALOG_REQ_PATH)));
  }
}
