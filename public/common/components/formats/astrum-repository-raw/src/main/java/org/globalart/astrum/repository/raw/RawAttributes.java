
package org.globalart.astrum.repository.raw;

import java.util.List;

import org.globalart.astrum.swagger.SwaggerEditionVisibility;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonFilter;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonInclude.Include;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;
import jakarta.validation.constraints.NotEmpty;

@JsonFilter(SwaggerEditionVisibility.NAME)
public class RawAttributes
{
  @SwaggerEditionVisibility(cloud = false, note = "Remove content-disposition from cloud")
  @Schema(description = "Content Disposition", example = "ATTACHMENT")
  @NotEmpty
  private final String contentDisposition;

  @Schema(description = "Whether to forward query parameters to the upstream repository",
      example = "true")
  @JsonInclude(Include.NON_NULL)
  private final Boolean forwardQueryParameters;

  @Schema(description = "List of query parameter names to exclude from forwarding (case-insensitive)",
      example = "[\"apiKey\", \"token\"]")
  @JsonInclude(Include.NON_EMPTY)
  private final List<String> excludedQueryParameters;

  @JsonCreator
  public RawAttributes(
      @JsonProperty("contentDisposition") final String contentDisposition,
      @JsonProperty("forwardQueryParameters") final Boolean forwardQueryParameters,
      @JsonProperty("excludedQueryParameters") final List<String> excludedQueryParameters)
  {
    this.contentDisposition = contentDisposition;
    this.forwardQueryParameters = forwardQueryParameters;
    this.excludedQueryParameters = excludedQueryParameters;
  }

  public RawAttributes(final String contentDisposition) {
    this(contentDisposition, null, null);
  }

  public String getContentDisposition() {
    return contentDisposition;
  }

  public Boolean getForwardQueryParameters() {
    return forwardQueryParameters;
  }

  public List<String> getExcludedQueryParameters() {
    return excludedQueryParameters;
  }
}
