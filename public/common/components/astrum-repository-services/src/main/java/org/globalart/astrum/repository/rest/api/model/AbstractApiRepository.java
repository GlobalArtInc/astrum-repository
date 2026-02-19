
package org.globalart.astrum.repository.rest.api.model;

import jakarta.validation.constraints.NotNull;
import jakarta.validation.constraints.Pattern;

import org.globalart.astrum.validation.constraint.NamePatternConstants;

import io.swagger.v3.oas.annotations.media.Schema;
import jakarta.validation.constraints.NotEmpty;

/**
 * REST API model of properties common to all repository types & formats.
 *
 * @since 3.20
 */
public abstract class AbstractApiRepository
{
  @Schema(description = "A unique identifier for this repository", example = "internal")
  @Pattern(regexp = NamePatternConstants.REGEX, message = NamePatternConstants.MESSAGE)
  @NotEmpty
  protected String name;

  @Schema(description = "Component format held in this repository", example = "npm")
  @NotEmpty
  protected String format;

  @Schema(description = "Controls if deployments of and updates to artifacts are allowed", example = "hosted")
  @NotEmpty
  protected String type;

  @Schema(description = "URL to the repository")
  protected String url;

  @Schema(description = "Whether this repository accepts incoming requests", example = "true")
  @NotNull
  protected Boolean online;

  public AbstractApiRepository(
      final String name,
      final String format,
      final String type,
      final String url,
      final Boolean online)
  {
    this.name = name;
    this.format = format;
    this.type = type;
    this.url = url;
    this.online = online;
  }

  public String getName() {
    return name;
  }

  public String getFormat() {
    return format;
  }

  public String getType() {
    return type;
  }

  public Boolean getOnline() {
    return online;
  }

  public String getUrl() {
    return url;
  }
}
