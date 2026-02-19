
package org.globalart.astrum.repository.rest.api;

import java.io.IOException;
import java.util.Set;

import com.fasterxml.jackson.core.JsonParser;
import com.fasterxml.jackson.databind.DeserializationContext;
import com.fasterxml.jackson.databind.JsonDeserializer;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Custom {@link JsonDeserializer} for the {@link ComponentXO} to handle the decorator approach
 *
 * @since 3.8
 */
public class ComponentXODeserializer
    extends JsonDeserializer<ComponentXO>
{
  private final ComponentXOFactory componentXOFactory;

  private final ObjectMapper objectMapper;

  private final Set<ComponentXODeserializerExtension> componentXODeserializerExtensions;

  public ComponentXODeserializer(
      final ComponentXOFactory componentXOFactory,
      final ObjectMapper objectMapper,
      final Set<ComponentXODeserializerExtension> componentXODeserializerExtensions)
  {
    this.componentXOFactory = checkNotNull(componentXOFactory);
    this.objectMapper = checkNotNull(objectMapper);
    this.componentXODeserializerExtensions = checkNotNull(componentXODeserializerExtensions);
  }

  @Override
  public ComponentXO deserialize(final JsonParser jsonParser, final DeserializationContext ctxt) throws IOException {
    JsonNode jsonNode = jsonParser.getCodec().readTree(jsonParser);

    ComponentXO componentXO = componentXOFactory.createComponentXO();

    // update the fresh ComponentXO with its own properties
    objectMapper.readerForUpdating(componentXO).readValue(jsonNode);

    // allow each extension to update its properties
    for (ComponentXODeserializerExtension componentXODeserializerExtension : componentXODeserializerExtensions) {
      componentXO = componentXODeserializerExtension.updateComponentXO(componentXO, jsonNode);
    }

    return componentXO;
  }
}
