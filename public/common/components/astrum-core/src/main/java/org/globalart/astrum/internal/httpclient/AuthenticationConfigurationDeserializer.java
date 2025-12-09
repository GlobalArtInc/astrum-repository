
package org.globalart.astrum.internal.httpclient;

import java.io.IOException;

import org.globalart.astrum.crypto.secrets.Secret;
import org.globalart.astrum.httpclient.config.AuthenticationConfiguration;
import org.globalart.astrum.httpclient.config.BearerTokenAuthenticationConfiguration;
import org.globalart.astrum.httpclient.config.NtlmAuthenticationConfiguration;
import org.globalart.astrum.httpclient.config.UsernameAuthenticationConfiguration;

import com.fasterxml.jackson.core.JsonParser;
import com.fasterxml.jackson.databind.DeserializationContext;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.deser.std.StdDeserializer;
import com.fasterxml.jackson.databind.jsontype.TypeDeserializer;

import static com.google.common.base.Preconditions.checkState;
import static org.globalart.astrum.httpclient.config.AuthenticationConfiguration.TYPES;

/**
 * {@link AuthenticationConfiguration} deserializer.
 *
 * Determines the instance class by the {@code type} field and decrypts sensitive data.
 *
 * @since 3.0
 */
public class AuthenticationConfigurationDeserializer
    extends StdDeserializer<AuthenticationConfiguration>
{

  public AuthenticationConfigurationDeserializer() {
    super(AuthenticationConfiguration.class);
  }

  @Override
  public AuthenticationConfiguration deserialize(
      final JsonParser parser,
      final DeserializationContext context) throws IOException
  {
    return deserialize(parser);
  }

  @Override
  public AuthenticationConfiguration deserializeWithType(
      final JsonParser parser,
      final DeserializationContext context,
      final TypeDeserializer typeDeserializer) throws IOException
  {
    return deserialize(parser);
  }

  private AuthenticationConfiguration deserialize(final JsonParser parser) throws IOException {
    JsonNode node = parser.readValueAsTree();
    String typeName = node.get("type").textValue();
    Class<? extends AuthenticationConfiguration> type = TYPES.get(typeName);
    checkState(type != null, "Unknown %s type: %s", AuthenticationConfiguration.class.getSimpleName(), typeName);
    AuthenticationConfiguration configuration = parser.getCodec().treeToValue(node, type);
    configuration.setPreemptive(configuration.isPreemptive());
    if (UsernameAuthenticationConfiguration.class.equals(type)) {
      UsernameAuthenticationConfiguration upc = (UsernameAuthenticationConfiguration) configuration;
      upc.setUsername(upc.getUsername());
    }
    else if (NtlmAuthenticationConfiguration.class.equals(type)) {
      NtlmAuthenticationConfiguration ntc = (NtlmAuthenticationConfiguration) configuration;
      ntc.setUsername(ntc.getUsername());
      ntc.setDomain(ntc.getDomain());
      ntc.setHost(ntc.getHost());
    }
    else if (BearerTokenAuthenticationConfiguration.class.equals(type)) {
      // Handle backward compatibility: old versions used "bearerToken", new versions use "bearerTokenId"
      BearerTokenAuthenticationConfiguration btc = (BearerTokenAuthenticationConfiguration) configuration;
      if (btc.getBearerToken() == null) {
        JsonNode bearerTokenNode = node.has("bearerToken") ? node.get("bearerToken") : null;
        if (bearerTokenNode != null && !bearerTokenNode.isNull()) {
          Secret secret = parser.getCodec().treeToValue(bearerTokenNode, Secret.class);
          btc.setBearerToken(secret);
        }
      }
    }
    return configuration;
  }
}
