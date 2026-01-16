
package org.globalart.astrum.swagger.internal;

import org.globalart.astrum.swagger.SwaggerContributor;

import io.swagger.v3.oas.models.Components;
import io.swagger.v3.oas.models.OpenAPI;
import io.swagger.v3.oas.models.media.Schema;
import io.swagger.v3.oas.models.media.StringSchema;
import org.springframework.stereotype.Component;

/**
 * SwaggerContributor that post-processes the OpenAPI model to fix the InputStream schema.
 * Changes the InputStream schema from type: object to type: string, format: binary.
 * This fixes all endpoints that reference #/components/schemas/InputStream (e.g., POST /v1/system/license).
 *
 * <p>
 * ASTRUM-46395: migrated from Swagger 1.x to OpenAPI 3.x. The {@code definitions} map at the top
 * level was relocated under {@code components.schemas} in OpenAPI 3.0.
 */
@Component
public class InputStreamSwaggerContributor
    implements SwaggerContributor
{
  @Override
  public void contribute(final OpenAPI openApi) {
    Components components = openApi.getComponents();
    if (components == null) {
      return;
    }
    if (components.getSchemas() == null) {
      return;
    }
    if (!components.getSchemas().containsKey("InputStream")) {
      return;
    }
    Schema<?> binarySchema = new StringSchema().format("binary");
    components.getSchemas().put("InputStream", binarySchema);
  }
}
