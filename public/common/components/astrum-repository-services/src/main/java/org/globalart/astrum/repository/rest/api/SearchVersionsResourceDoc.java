
package org.globalart.astrum.repository.rest.api;

import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.Parameter;
import io.swagger.v3.oas.annotations.media.Content;
import io.swagger.v3.oas.annotations.media.Schema;
import io.swagger.v3.oas.annotations.responses.ApiResponse;
import io.swagger.v3.oas.annotations.responses.ApiResponses;
import io.swagger.v3.oas.annotations.tags.Tag;

/**
 * OpenAPI documentation for {@link org.globalart.astrum.repository.rest.internal.resources.SearchVersionsResource}.
 */
@Tag(name = "Search")
public interface SearchVersionsResourceDoc
{
  String FORMAT_DESCRIPTION = "Component format (e.g., maven2, npm, nuget)";

  String GROUP_DESCRIPTION = "Component namespace or group; omit for formats without one";

  String NAME_DESCRIPTION = "Component name";

  String VERSION_DESCRIPTION = "Filter to versions containing this substring";

  String PAGE_DESCRIPTION = "Zero-based page index";

  String SIZE_DESCRIPTION = "Page size, 1 to 250";

  String SORT_DESCRIPTION = "Sort key: version, lastUpdated, or repositories";

  String DIRECTION_DESCRIPTION = "Sort direction: asc or desc";

  @Operation(summary = "Browse the distinct versions of a component")
  @ApiResponses(value = {
      @ApiResponse(responseCode = "200", description = "A page of distinct versions",
          content = @Content(schema = @Schema(implementation = ComponentVersionsPageXO.class))),
      @ApiResponse(responseCode = "400", description = "Invalid parameter value"),
      @ApiResponse(responseCode = "401", description = "Authentication required"),
  })
  ComponentVersionsPageXO getVersions(
      @Parameter(description = FORMAT_DESCRIPTION, required = true) String format,
      @Parameter(description = GROUP_DESCRIPTION) String group,
      @Parameter(description = NAME_DESCRIPTION, required = true) String name,
      @Parameter(description = VERSION_DESCRIPTION) String version,
      @Parameter(description = PAGE_DESCRIPTION) int page,
      @Parameter(description = SIZE_DESCRIPTION) int size,
      @Parameter(description = SORT_DESCRIPTION,
          schema = @Schema(allowableValues = {"version", "lastUpdated", "repositories"})) String sort,
      @Parameter(description = DIRECTION_DESCRIPTION,
          schema = @Schema(allowableValues = {"asc", "desc"})) String direction);
}
