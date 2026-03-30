
package org.globalart.astrum.repository.npm.internal;

import java.io.IOException;
import java.util.List;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.json.UntypedObjectDeserializerSerializer;
import org.globalart.astrum.repository.npm.internal.NpmFieldFactory.SkipObjectNpmFieldDeserializer;

import com.fasterxml.jackson.core.JsonGenerator;
import com.fasterxml.jackson.core.JsonParser;
import com.fasterxml.jackson.databind.DeserializationContext;
import com.fasterxml.jackson.databind.deser.std.UntypedObjectDeserializer;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * {@link UntypedObjectDeserializer} that is NPM specific by instantly writing out to the provided generator,
 * rather then maintaining references in a map until all values have been deserialized.
 *
 * @since 3.16
 */
public class NpmUntypedObjectDeserializerSerializer
    extends UntypedObjectDeserializerSerializer
{
  private final List<NpmFieldMatcher> matchers;

  public NpmUntypedObjectDeserializerSerializer(final JsonGenerator generator,
                                                final List<NpmFieldMatcher> matchers)
  {
    super(generator);
    this.matchers = checkNotNull(matchers);
  }

  /**
   * Overwritten from {@link UntypedObjectDeserializer} allowing the deserialized JSON to be streamed out directly and
   * preventing the deserialized object from being kept in memory.
   *
   * @param parser  {@link JsonParser}
   * @param context {@link DeserializationContext}
   * @return an {@link Object} of any type, if needing to temporary keep it in memory, otherwise null.
   * @throws IOException if unable to properly read and parse given {@link JsonGenerator}.
   */
  @Override
  @Nullable
  public Object deserialize(final JsonParser parser, final DeserializationContext context) throws IOException {
    String fieldName = parser.getCurrentName();

    for (NpmFieldMatcher matcher : matchers) {
      if (matcher.matches(parser) && matcher.allowDeserializationOnMatched()) {
        // first matcher wins
        NpmFieldDeserializer deserializer = matcher.getDeserializer();
        if(deserializer instanceof SkipObjectNpmFieldDeserializer) {
          // Matchers with values of type array/object will fail getting the default value. Its an edge case not yet
          // supported or needed. If we're skipping the object then we don't need the default value
          return deserializer.deserialize(fieldName, null, parser, context, generator);
        }
        Object defaultValue = defaultValueDeserialize(parser, context);
        return deserializer.deserialize(fieldName, defaultValue, parser, context, generator);
      }
    }

    return defaultDeserialize(fieldName, parser, context);
  }
}
