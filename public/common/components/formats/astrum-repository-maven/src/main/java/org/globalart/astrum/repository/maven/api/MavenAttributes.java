
package org.globalart.astrum.repository.maven.api;

import org.globalart.astrum.swagger.SwaggerEditionVisibility;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonFilter;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;
import jakarta.validation.constraints.NotEmpty;
import jakarta.validation.constraints.Pattern;

/**
 * REST API model for describing maven specific repository properties.
 *
 * @since 3.20
 */
@JsonFilter(SwaggerEditionVisibility.NAME)
public class MavenAttributes
{
  @Schema(description = "What type of artifacts does this repository store?",
      example = "MIXED")
  @NotEmpty
  @Pattern(regexp = "RELEASE|SNAPSHOT|MIXED", message = "must be one of RELEASE, SNAPSHOT, MIXED")
  protected final String versionPolicy;

  @Schema(description = "Validate that all paths are maven artifact or metadata paths",
      example = "STRICT")
  @NotEmpty
  @Pattern(regexp = "STRICT|PERMISSIVE", message = "must be one of STRICT, PERMISSIVE")
  protected final String layoutPolicy;

  @Schema(description = "Content Disposition", example = "ATTACHMENT")
  @SwaggerEditionVisibility(cloud = false, note = "Remove content-disposition from cloud")
  @Pattern(regexp = "INLINE|ATTACHMENT", message = "must be one of INLINE, ATTACHMENT")
  private final String contentDisposition;

  @JsonCreator
  public MavenAttributes(
      @JsonProperty("versionPolicy") final String versionPolicy,
      @JsonProperty("layoutPolicy") final String layoutPolicy,
      @JsonProperty("contentDisposition") final String contentDisposition)
  {
    this.versionPolicy = versionPolicy;
    this.layoutPolicy = layoutPolicy;
    this.contentDisposition = contentDisposition;
  }

  public String getVersionPolicy() {
    return versionPolicy;
  }

  public String getLayoutPolicy() {
    return layoutPolicy;
  }

  public String getContentDisposition() {
    return contentDisposition;
  }
}
