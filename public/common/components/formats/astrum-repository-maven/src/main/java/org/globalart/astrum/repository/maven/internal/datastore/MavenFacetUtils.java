
package org.globalart.astrum.repository.maven.internal.datastore;

import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.maven.internal.Maven2Format;

import static org.globalart.astrum.repository.maven.internal.Attributes.P_BASE_VERSION;
import static org.globalart.astrum.repository.maven.internal.Constants.SNAPSHOT_VERSION_SUFFIX;

/**
 * Maven facet utilities for datastore implementation.
 *
 * @since 3.31
 */
public final class MavenFacetUtils
{
  private MavenFacetUtils() {
  }

  /**
   * Is a given Component a snapshot
   */
  public static boolean isSnapshot(final Component component) {
    String baseVersion = (String) component.attributes().child(Maven2Format.NAME).get(P_BASE_VERSION);
    return baseVersion != null && baseVersion.endsWith(SNAPSHOT_VERSION_SUFFIX);
  }
}
