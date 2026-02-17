
package org.globalart.astrum.repository.application.scan;

import java.util.Map;

import org.globalart.astrum.common.collect.NestedAttributesMap;

public interface AttributesCoordinatesAdapter
{
  Map<String, String> toCoordinates(final NestedAttributesMap attributes);

  Map<String, String> toArtifactCoordinates(final Map<String, String> coordinates);
}
