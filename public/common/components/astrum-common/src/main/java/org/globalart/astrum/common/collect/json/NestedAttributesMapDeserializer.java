
package org.globalart.astrum.common.collect.json;

import java.io.IOException;
import java.util.Map;

import org.globalart.astrum.common.collect.NestedAttributesMap;

import com.fasterxml.jackson.core.JsonParser;
import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.DeserializationContext;
import com.fasterxml.jackson.databind.JsonDeserializer;

/**
 * Custom Jackson deserializer for {@link NestedAttributesMap} that deserializes from a simple map structure,
 * complementing the {@link NestedAttributesMapSerializer}.
 *
 * @since 3.75
 */
public class NestedAttributesMapDeserializer
    extends JsonDeserializer<NestedAttributesMap>
{
  private static final TypeReference<Map<String, Object>> MAP_TYPE_REF = new TypeReference<Map<String, Object>>()
  {
  };

  @Override
  public NestedAttributesMap deserialize(
      final JsonParser p,
      final DeserializationContext ctxt) throws IOException
  {
    if (p.currentToken() == null || p.currentToken().isScalarValue() && p.getText() == null) {
      return null;
    }

    // Deserialize as a Map and wrap in a NestedAttributesMap
    Map<String, Object> backing = p.readValueAs(MAP_TYPE_REF);
    return new NestedAttributesMap("attributes", backing);
  }
}
