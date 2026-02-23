
package org.globalart.astrum.repository.rest.internal.resources;

import java.util.List;
import java.util.stream.Collectors;

import org.globalart.astrum.repository.rest.api.SuggestXO;
import org.globalart.astrum.repository.search.ComponentSearchResult;
import org.globalart.astrum.repository.search.SearchRequest;
import org.globalart.astrum.repository.search.SearchResponse;
import org.globalart.astrum.repository.search.SearchService;
import org.globalart.astrum.rest.Resource;

import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.Parameter;
import io.swagger.v3.oas.annotations.media.ArraySchema;
import io.swagger.v3.oas.annotations.media.Content;
import io.swagger.v3.oas.annotations.media.Schema;
import io.swagger.v3.oas.annotations.responses.ApiResponse;
import io.swagger.v3.oas.annotations.responses.ApiResponses;
import io.swagger.v3.oas.annotations.tags.Tag;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.QueryParam;
import org.apache.shiro.authz.annotation.RequiresUser;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import static org.globalart.astrum.common.app.FeatureFlags.PREVIEW_UI_SETTINGS_ENABLED;
import static org.globalart.astrum.rest.APIConstants.V1_API_PREFIX;

/**
 * Lightweight search suggestions endpoint for autocomplete/typeahead functionality.
 * Returns minimal component data to optimize response size and latency.
 */
@Component
@ConditionalOnProperty(name = PREVIEW_UI_SETTINGS_ENABLED, havingValue = "true", matchIfMissing = true)
@Path(SuggestResource.RESOURCE_URI)
@Produces(APPLICATION_JSON)
@Consumes(APPLICATION_JSON)
@Tag(name = "Search", description = "Search suggestions for autocomplete")
public class SuggestResource
    implements Resource
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final String RESOURCE_URI = V1_API_PREFIX + "/search/suggest";

  private static final int DEFAULT_LIMIT = 10;

  private static final int MAX_LIMIT = 20;

  private static final int MIN_QUERY_LENGTH = 2;

  private final SearchService searchService;

  @Autowired
  public SuggestResource(final SearchService searchService) {
    this.searchService = checkNotNull(searchService);
  }

  @RequiresUser
  @GET
  @Operation(summary = "Get search suggestions for autocomplete",
      description = "Returns lightweight component suggestions matching the query. " +
          "Optimized for fast typeahead with minimal response payload.")
  @ApiResponses({
      @ApiResponse(responseCode = "200", description = "Suggestions returned successfully",
          content = @Content(array = @ArraySchema(schema = @Schema(implementation = SuggestXO.class)))),
      @ApiResponse(responseCode = "400", description = "Query too short (minimum 2 characters)")
  })
  public List<SuggestXO> suggest(
      @Parameter(description = "Search query (minimum 2 characters)",
          required = true) @QueryParam("q") final String query,

      @Parameter(description = "Filter by format (e.g., maven2, npm, pypi)") @QueryParam("format") final String format,

      @Parameter(
          description = "Maximum number of suggestions (default: 10, max: 20)") @QueryParam("limit") final Integer limit)
  {
    // Validate query length
    if (query == null || query.trim().length() < MIN_QUERY_LENGTH) {
      return List.of(); // Return empty for short queries
    }

    // Determine limit
    int effectiveLimit = DEFAULT_LIMIT;
    if (limit != null && limit > 0) {
      effectiveLimit = Math.min(limit, MAX_LIMIT);
    }

    // Build search request - NO assets, minimal data
    SearchRequest.Builder requestBuilder = SearchRequest.builder()
        .searchFilter("keyword", query.trim())
        .limit(effectiveLimit);
    // Note: NOT calling .includeAssets() - this keeps response small

    // Add format filter if specified
    if (format != null && !format.isBlank()) {
      requestBuilder.searchFilter("format", format.trim());
    }

    SearchRequest request = requestBuilder.build();
    SearchResponse response = searchService.search(request);

    // Map to lightweight DTOs
    return response.getSearchResults()
        .stream()
        .map(this::toSuggestXO)
        .collect(Collectors.toList());
  }

  private SuggestXO toSuggestXO(final ComponentSearchResult result) {
    return new SuggestXO(
        result.getId(),
        result.getName(),
        result.getGroup(),
        result.getVersion(),
        result.getFormat(),
        result.getRepositoryName());
  }
}
