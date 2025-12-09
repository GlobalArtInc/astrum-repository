
package org.globalart.astrum.internal.httpclient;

import java.io.IOException;

import org.globalart.astrum.common.time.Time;

import com.fasterxml.jackson.core.JsonGenerator;
import com.fasterxml.jackson.databind.SerializerProvider;
import com.fasterxml.jackson.databind.ser.std.StdSerializer;

/**
 * {@link Time} in seconds serializer.
 *
 * @since 3.0
 */
public class SecondsSerializer
    extends StdSerializer<Time>
{

  public SecondsSerializer() {
    super(Time.class);
  }

  @Override
  public void serialize(
      final Time value,
      final JsonGenerator jgen,
      final SerializerProvider provider) throws IOException
  {
    jgen.writeNumber(value.toSecondsI());
  }
}
