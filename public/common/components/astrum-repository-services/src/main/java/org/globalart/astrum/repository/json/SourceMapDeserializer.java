
package org.globalart.astrum.repository.json;

import java.util.Set;

import com.fasterxml.jackson.databind.JavaType;
import com.fasterxml.jackson.databind.JsonDeserializer;
import com.fasterxml.jackson.databind.KeyDeserializer;
import com.fasterxml.jackson.databind.deser.NullValueProvider;
import com.fasterxml.jackson.databind.deser.ValueInstantiator;
import com.fasterxml.jackson.databind.deser.std.MapDeserializer;
import com.fasterxml.jackson.databind.jsontype.TypeDeserializer;

/**
 * Implementation of {@link MapDeserializer} that opens up the "from source" {@link MapDeserializer} constructor.
 *
 * @since 3.17
 */
public class SourceMapDeserializer
    extends MapDeserializer
{
  private SourceMapDeserializer(
      final MapDeserializer src,
      final KeyDeserializer keyDeser,
      final JsonDeserializer<Object> valueDeser,
      final TypeDeserializer valueTypeDeser,
      final NullValueProvider nuller,
      final Set<String> ignorable)
  {
    super(src, keyDeser, valueDeser, valueTypeDeser, nuller, ignorable);
  }

  public static SourceMapDeserializer of(
      final JavaType mapType,
      final ValueInstantiator valueInstantiator,
      final JsonDeserializer<Object> deserializer)
  {
    return of(new MapDeserializer(mapType, valueInstantiator, null, deserializer, null));
  }

  public static SourceMapDeserializer of(final MapDeserializer mapDeserializer) {
    JsonDeserializer<Object> deserializer = mapDeserializer.getContentDeserializer();
    return new SourceMapDeserializer(mapDeserializer, null, deserializer, null, deserializer, null);
  }
}
