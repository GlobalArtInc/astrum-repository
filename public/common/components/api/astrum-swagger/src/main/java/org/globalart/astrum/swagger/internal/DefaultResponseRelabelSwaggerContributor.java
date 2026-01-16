
package org.globalart.astrum.swagger.internal;

import org.globalart.astrum.swagger.SwaggerContributor;

import io.swagger.v3.oas.models.OpenAPI;
import io.swagger.v3.oas.models.Operation;
import io.swagger.v3.oas.models.responses.ApiResponse;
import io.swagger.v3.oas.models.responses.ApiResponses;
import org.springframework.stereotype.Component;

/**
 * Restores the {@code 200} status for operations whose typed body schema was filed under the generic
 * {@code default} response key by the OpenAPI 3 (swagger-jaxrs2) scanner (ASTRUM-54046, shape B).
 *
 * <p>
 * Only operations whose <em>sole</em> response is a {@code default} carrying content are relabelled; a
 * {@code default} without content (e.g. {@code Response}-typed operations, tracked in ASTRUM-54063) or a
 * {@code default} alongside other status codes is left untouched. The relabel is idempotent.
 *
 * <p>
 * <b>Ordering assumption:</b> the {@code responses.size() == 1} guard is load-bearing. It relies on this
 * contributor observing the scanner's raw output, where a shape-B operation has only the {@code default}
 * key. No {@link SwaggerContributor} contributed before this one adds a status code to such operations
 * today; if one ever does (making {@code size() > 1}), the operation would be silently skipped here and
 * would need an explicit {@code @ApiResponse(200)} instead.
 */
@Component
public class DefaultResponseRelabelSwaggerContributor
    implements SwaggerContributor
{
  private static final String DEFAULT_KEY = "default";

  private static final String DEFAULT_RESPONSE_DESCRIPTION = "default response";

  @Override
  public void contribute(final OpenAPI openApi) {
    if (openApi.getPaths() == null) {
      return;
    }
    openApi.getPaths().values().forEach(pathItem -> pathItem.readOperations().forEach(this::relabel));
  }

  private void relabel(final Operation operation) {
    ApiResponses responses = operation.getResponses();
    if (responses == null || responses.size() != 1) {
      return;
    }
    ApiResponse defaultResponse = responses.get(DEFAULT_KEY);
    if (defaultResponse == null || !hasContent(defaultResponse)) {
      return;
    }
    responses.remove(DEFAULT_KEY);
    String description = defaultResponse.getDescription();
    if (description == null || description.isBlank()
        || DEFAULT_RESPONSE_DESCRIPTION.equals(description)) {
      defaultResponse.setDescription("successful operation");
    }
    responses.addApiResponse("200", defaultResponse);
  }

  private boolean hasContent(final ApiResponse response) {
    return response.getContent() != null && !response.getContent().isEmpty();
  }
}
