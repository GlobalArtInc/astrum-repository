
package org.globalart.astrum.repository.rest.internal.resources.doc;

import java.util.List;

import jakarta.validation.Valid;

import org.globalart.astrum.repository.rest.api.ContentSelectorApiCreateRequest;
import org.globalart.astrum.repository.rest.api.ContentSelectorApiResponse;
import org.globalart.astrum.repository.rest.api.ContentSelectorApiUpdateRequest;

import io.swagger.v3.oas.annotations.tags.Tag;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.Parameter;
import io.swagger.v3.oas.annotations.responses.ApiResponse;
import io.swagger.v3.oas.annotations.responses.ApiResponses;
import io.swagger.v3.oas.annotations.media.Content;
import io.swagger.v3.oas.annotations.media.Schema;
import io.swagger.v3.oas.annotations.media.ArraySchema;

/**
 * Swagger documentation for {@link org.globalart.astrum.repository.rest.internal.resources.ContentSelectorsApiResource}
 *
 * @since 3.19
 */
@Tag(name = "Content selectors")
public interface ContentSelectorsResourceDoc
{
  String NAME_DESCRIPTION = "The content selector name cannot be changed after creation";

  String TYPE_DESCRIPTION = "The type of content selector the backend is using";

  String TYPE_ALLOWED_VALUES = "csel, jexl";

  String TYPE_NOTES = "All new content selectors will be created as csel selectors, jexl selectors are deprecated";

  String DESCRIPTION_DESCRIPTION = "A human-readable description";

  String EXPRESSION_DESCRIPTION = "The expression used to identify content";

  String EXPRESSION_EXAMPLE = "format == \"maven2\" and path =^ \"/org/globalart/astrum\"";

  String EXPRESSION_NOTES = "See http://links.globalart.dev/products/astrum/selectors/docs for more details";

  @Operation(summary = "List content selectors")
  @ApiResponses({
      @ApiResponse(responseCode = "200", description = "successful operation",
          content = @Content(
              array = @ArraySchema(schema = @Schema(implementation = ContentSelectorApiResponse.class)))),
      @ApiResponse(responseCode = "403", description = "Insufficient permissions to read content selectors")
  })
  List<ContentSelectorApiResponse> getContentSelectors();

  @Operation(summary = "Create a new content selector")
  @ApiResponses({
      @ApiResponse(responseCode = "204", description = "Content selector successfully created"),
      @ApiResponse(responseCode = "400", description = "Invalid request"),
      @ApiResponse(responseCode = "403", description = "Insufficient permissions to create content selectors")
  })
  void createContentSelector(@Valid final ContentSelectorApiCreateRequest request);

  @Operation(summary = "Get a content selector by name")
  @ApiResponses({
      @ApiResponse(responseCode = "200", description = "successful operation",
          content = @Content(schema = @Schema(implementation = ContentSelectorApiResponse.class))),
      @ApiResponse(responseCode = "403", description = "Insufficient permissions to read the content selector")
  })
  ContentSelectorApiResponse getContentSelector(
      @Parameter(required = true, description = "The content selector name") final String name);

  @Operation(summary = "Update a content selector")
  @ApiResponses({
      @ApiResponse(responseCode = "204", description = "Content selector updated successfully"),
      @ApiResponse(responseCode = "400", description = "Invalid request"),
      @ApiResponse(responseCode = "403", description = "Insufficient permissions to update the content selector")
  })
  void updateContentSelector(
      @Parameter(required = true, description = "The content selector name") final String name,
      @Valid final ContentSelectorApiUpdateRequest contentSelector);

  @Operation(summary = "Delete a content selector")
  @ApiResponses({
      @ApiResponse(responseCode = "204", description = "Content selector deleted successfully"),
      @ApiResponse(responseCode = "400", description = "Invalid request"),
      @ApiResponse(responseCode = "403", description = "Insufficient permissions to delete the content selector")
  })
  void deleteContentSelector(final String name);
}
