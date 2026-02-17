
package org.globalart.astrum.repository.application.scan;

import java.util.Map;
import org.globalart.astrum.common.collect.NestedAttributesMap;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.context.annotation.Primary;
import org.springframework.stereotype.Component;

@Primary
@Component
@Qualifier(DefaultAttributesCoordinatesAdapter.DEFAULT)
public class DefaultAttributesCoordinatesAdapter
    implements AttributesCoordinatesAdapter
{
  public static final String DEFAULT = "default";

  @Override
  public Map<String, String> toCoordinates(final NestedAttributesMap attributes) {
    return null;
  }

  @Override
  public Map<String, String> toArtifactCoordinates(final Map<String, String> coordinates) {
    return coordinates;
  }
}
