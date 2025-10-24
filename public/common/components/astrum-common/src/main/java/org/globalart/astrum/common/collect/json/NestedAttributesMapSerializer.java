
package org.globalart.astrum.common.collect.json;

import java.io.IOException;

import org.globalart.astrum.common.collect.NestedAttributesMap;

import com.fasterxml.jackson.core.JsonGenerator;
import com.fasterxml.jackson.databind.JsonSerializer;
import com.fasterxml.jackson.databind.SerializerProvider;

/**
 * Custom Jackson serializer for {@link NestedAttributesMap} that serializes only the backing map contents,
 * avoiding the internal structure (parent, key fields) that causes JSON parsing issues during configuration export.
 *
 * @since 3.75
 */
public class NestedAttributesMapSerializer
    extends JsonSerializer<NestedAttributesMap>
{
  @Override
  public void serialize(
      final NestedAttributesMap value,
      final JsonGenerator gen,
      final SerializerProvider serializers) throws IOException
  {
    if (value == null) {
      gen.writeNull();
    }
    else {
      // Serialize only the backing map, not the parent/key metadata
      gen.writeObject(value.backing());
    }
  }
}
