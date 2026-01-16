
package org.globalart.astrum.swagger.internal;

import org.globalart.astrum.rest.Component;

/**
 * Siesta-managed {@link io.swagger.v3.jaxrs2.SwaggerSerializers}.
 *
 * <p>
 * ASTRUM-46395: migrated from Swagger 1.x ({@code io.swagger.jaxrs.listing.SwaggerSerializers})
 * to OpenAPI 3.x. The new base class outputs OpenAPI 3.0 JSON/YAML rather than Swagger 2.0 JSON;
 * customers consuming the {@code /v3/api-docs} endpoint will see this shape change. Pretty-printing
 * is configured at the ObjectMapper level in OpenAPI 3.x; we no longer call setPrettyPrint().
 */
@org.springframework.stereotype.Component
public class SwaggerSerializers
    extends io.swagger.v3.jaxrs2.SwaggerSerializers
    implements Component
{
  public SwaggerSerializers() { // NOSONAR
    // OpenAPI 3.x SwaggerSerializers always pretty-prints by default; no toggle needed.
  }
}
