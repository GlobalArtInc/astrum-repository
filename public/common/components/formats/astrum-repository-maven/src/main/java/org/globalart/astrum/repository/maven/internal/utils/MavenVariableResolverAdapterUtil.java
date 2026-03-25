
package org.globalart.astrum.repository.maven.internal.utils;

import java.util.HashMap;
import java.util.Map;

import org.globalart.astrum.repository.maven.MavenPath.Coordinates;

import static org.apache.commons.lang3.StringUtils.EMPTY;

/**
 * Contains utilities shared by both the
 * {@link org.globalart.astrum.repository.maven.internal.orient.OrientMavenVariableResolverAdapter}
 * and {@link org.globalart.astrum.content.maven.internal.MavenVariableResolverAdapter}
 *
 * @since 3.25
 */
public class MavenVariableResolverAdapterUtil
{
  private static final String GROUP_ID = "groupId";

  private static final String ARTIFACT_ID = "artifactId";

  private static final String VERSION = "version";

  private static final String EXTENSION = "extension";

  private static final String CLASSIFIER = "classifier";

  private MavenVariableResolverAdapterUtil() {
  }

  public static Map<String, String> createCoordinateMap(Coordinates coordinates) {
    Map<String, String> coordMap = new HashMap<>();
    coordMap.put(GROUP_ID, coordinates.getGroupId());
    coordMap.put(ARTIFACT_ID, coordinates.getArtifactId());
    coordMap.put(VERSION, coordinates.getBaseVersion());
    coordMap.put(EXTENSION, coordinates.getExtension());
    coordMap.put(CLASSIFIER, coordinates.getClassifier() == null ? EMPTY : coordinates.getClassifier());
    return coordMap;
  }
}
