
package org.globalart.astrum.repository.maven.internal.attributes;

import java.util.Map;
import java.util.TreeMap;
import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.application.scan.AttributesCoordinatesAdapter;
import org.globalart.astrum.repository.maven.internal.Attributes;
import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

@Component
@Qualifier(Maven2Format.NAME)
public class Maven2AttributesCoordinatesAdapter
    implements AttributesCoordinatesAdapter
{
  private static final String WILDCARD = "*";

  @Override
  public Map<String, String> toCoordinates(final NestedAttributesMap attributes) {
    Map<String, String> coordinates = new TreeMap<>();
    coordinates.put(Attributes.P_GROUP_ID, attributes.get(Attributes.P_GROUP_ID, String.class));
    coordinates.put(Attributes.P_ARTIFACT_ID, attributes.get(Attributes.P_ARTIFACT_ID, String.class));
    coordinates.put(Attributes.P_VERSION, attributes.get(Attributes.P_VERSION, String.class));
    coordinates.put(Attributes.P_CLASSIFIER, attributes.get(Attributes.P_CLASSIFIER, String.class));
    coordinates.put(Attributes.P_EXTENSION, attributes.get(Attributes.P_EXTENSION, String.class));
    return coordinates;
  }

  @Override
  public Map<String, String> toArtifactCoordinates(final Map<String, String> coordinates) {
    Map<String, String> artifactCoordinates = new TreeMap<>(coordinates);
    artifactCoordinates.put(Attributes.P_CLASSIFIER, WILDCARD);
    artifactCoordinates.put(Attributes.P_EXTENSION, WILDCARD);
    return artifactCoordinates;
  }
}
