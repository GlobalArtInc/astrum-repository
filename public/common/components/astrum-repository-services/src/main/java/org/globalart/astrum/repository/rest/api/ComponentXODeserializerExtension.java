
package org.globalart.astrum.repository.rest.api;

import com.fasterxml.jackson.databind.JsonNode;

/**
 * Extension point for the {@link ComponentXODeserializer}
 *
 * @since 3.8
 */
public interface ComponentXODeserializerExtension
{
  /**
   * Update the given {@link ComponentXO} with the data in the {@link JsonNode}
   */
  ComponentXO updateComponentXO(final ComponentXO componentXO, final JsonNode jsonNode);
}
