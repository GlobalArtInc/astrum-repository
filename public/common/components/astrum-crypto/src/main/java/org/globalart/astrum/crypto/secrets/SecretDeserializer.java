
package org.globalart.astrum.crypto.secrets;

import java.io.IOException;

import com.fasterxml.jackson.core.JsonParser;
import com.fasterxml.jackson.core.JsonToken;
import com.fasterxml.jackson.databind.DeserializationContext;
import com.fasterxml.jackson.databind.deser.std.StdDeserializer;

/**
 * {@link Secret} deserializer
 * <p>
 * wraps the value coming from the json in a {@link Secret} object
 */
public class SecretDeserializer
    extends StdDeserializer<Secret>
{
  private final SecretsFactory secretsFactory;

  public SecretDeserializer(final SecretsFactory secretsFactory) {
    super(Secret.class);
    this.secretsFactory = secretsFactory;
  }

  @Override
  public Secret deserialize(
      final JsonParser jsonParser,
      final DeserializationContext deserializationContext) throws IOException
  {
    // if the value is null we return null
    if (jsonParser.getCurrentToken() == JsonToken.VALUE_NULL) {
      return null;
    }
    // use factory to parse it , in the worst case we have the legacy secret and decrypt will handle that
    return secretsFactory.from(jsonParser.getText());
  }
}
