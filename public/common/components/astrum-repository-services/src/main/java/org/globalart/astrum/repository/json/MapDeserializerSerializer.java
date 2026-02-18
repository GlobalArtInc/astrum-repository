
package org.globalart.astrum.repository.json;

import com.fasterxml.jackson.databind.deser.std.MapDeserializer;

/**
 * Class that acts as a {@link MapDeserializer} but allows the results to be serialized out right away by
 * using a {@link UntypedObjectDeserializerSerializer} for its deserialization of individual objects and arrays.
 */
public class MapDeserializerSerializer
    extends MapDeserializer
{
  private MapDeserializerSerializer(final MapDeserializer rootDeserializer) {
    super(rootDeserializer);
  }

  private MapDeserializer withResolved(final UntypedObjectDeserializerSerializer deserializerSerializer) {
    return withResolved(_keyDeserializer, _valueTypeDeserializer, deserializerSerializer, _nullProvider,
        _ignorableProperties);
  }

  public static MapDeserializer create(
      final MapDeserializer root,
      final UntypedObjectDeserializerSerializer deserializerSerializer)
  {
    return new MapDeserializerSerializer(root).withResolved(deserializerSerializer);
  }
}
