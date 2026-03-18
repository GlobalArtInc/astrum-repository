
package org.globalart.astrum.repository.raw.rest;

import java.util.List;

import org.globalart.astrum.repository.raw.ContentDisposition;
import org.globalart.astrum.swagger.SwaggerEditionVisibility;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonFilter;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;
import jakarta.validation.constraints.Size;

/**
 * REST API model of raw attributes for repositories API
 *
 * @since 3.25
 */
@JsonFilter(SwaggerEditionVisibility.NAME)
public class RawAttributes
{
  public static final String CONTENT_DISPOSITION = "contentDisposition";

  public static final String FORWARD_QUERY_PARAMETERS = "forwardQueryParameters";

  public static final String EXCLUDED_QUERY_PARAMETERS = "excludedQueryParameters";

  @Schema(description = "Content Disposition", example = "ATTACHMENT")
  @SwaggerEditionVisibility(cloud = false, note = "Hide content-disposition from cloud")
  private final ContentDisposition contentDisposition;

  @Schema(description = "Whether to forward query parameters to the upstream repository",
      example = "true")
  private final Boolean forwardQueryParameters;

  @Schema(
      description = "List of query parameter names to exclude from forwarding (case-insensitive). Maximum 100 entries.",
      example = "[\"apiKey\", \"token\"]")
  @Size(max = 100, message = "excludedQueryParameters may contain at most 100 entries")
  private final List<String> excludedQueryParameters;

  @JsonCreator
  public RawAttributes(
      @JsonProperty(CONTENT_DISPOSITION) final ContentDisposition contentDisposition,
      @JsonProperty(FORWARD_QUERY_PARAMETERS) final Boolean forwardQueryParameters,
      @JsonProperty(EXCLUDED_QUERY_PARAMETERS) final List<String> excludedQueryParameters)
  {
    this.contentDisposition = contentDisposition;
    this.forwardQueryParameters = forwardQueryParameters;
    this.excludedQueryParameters = excludedQueryParameters;
  }

  /**
   * Backward-compatible constructor for existing code that only sets contentDisposition
   */
  public RawAttributes(final ContentDisposition contentDisposition) {
    this(contentDisposition, null, null);
  }

  public ContentDisposition getContentDisposition() {
    return contentDisposition;
  }

  public Boolean getForwardQueryParameters() {
    return forwardQueryParameters;
  }

  public List<String> getExcludedQueryParameters() {
    return excludedQueryParameters;
  }
}
