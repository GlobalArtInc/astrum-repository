
package org.globalart.astrum.swagger;

import io.swagger.v3.oas.models.OpenAPI;

/**
 * Listener providing a hook for customizing the {@link OpenAPI} model.
 *
 * <p>
 * ASTRUM-46395: migrated from Swagger 1.x ({@code io.swagger.models.Swagger}) to OpenAPI 3.x
 * ({@code io.swagger.v3.oas.models.OpenAPI}). Implementing plugins must update their
 * {@link #contribute(OpenAPI)} signature accordingly.
 */
public interface SwaggerContributor
{
  /**
   * Called after JAX-RS resource has been scanned.
   *
   * @param openApi the OpenAPI definition
   */
  void contribute(OpenAPI openApi);
}
