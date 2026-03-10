
package org.globalart.astrum.supportzip.datastore;

import java.io.IOException;

import org.globalart.astrum.common.time.Time;

import com.fasterxml.jackson.core.JsonParser;
import com.fasterxml.jackson.databind.DeserializationContext;
import com.fasterxml.jackson.databind.deser.std.StdDeserializer;

/**
 * {@link Time} in seconds deserializer.
 *
 * @since 3.29
 */
public class SecondsDeserializer
    extends StdDeserializer<Time>
{
  private static final long serialVersionUID = -5331061510739363902L;

  public SecondsDeserializer() {
    super(Time.class);
  }

  @Override
  public Time deserialize(final JsonParser parser, final DeserializationContext context) throws IOException {
    return Time.seconds(parser.readValueAs(Long.class));
  }
}
