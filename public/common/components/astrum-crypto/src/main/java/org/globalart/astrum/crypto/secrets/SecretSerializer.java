
package org.globalart.astrum.crypto.secrets;

import java.io.IOException;

import com.fasterxml.jackson.core.JsonGenerator;
import com.fasterxml.jackson.databind.SerializerProvider;
import com.fasterxml.jackson.databind.ser.std.StdSerializer;

/**
 * {@link Secret} serializer
 * <p>
 * Stores the secret id if the system has migrated, otherwise stores the legacy encrypted value
 */
public class SecretSerializer
    extends StdSerializer<Secret>
{
  public SecretSerializer() {
    super(Secret.class);
  }

  @Override
  public void serialize(
      final Secret secret,
      final JsonGenerator jsonGenerator,
      final SerializerProvider serializerProvider) throws IOException
  {
    if (secret != null) {
      // if the secret is not null we just write the id, in the worst case will be the legacy value
      jsonGenerator.writeString(secret.getId());
    }
    else {
      // if the secret is null we write null
      jsonGenerator.writeNull();
    }
  }
}
