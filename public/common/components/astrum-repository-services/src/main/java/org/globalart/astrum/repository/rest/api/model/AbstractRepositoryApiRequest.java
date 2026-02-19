
package org.globalart.astrum.repository.rest.api.model;

import jakarta.validation.constraints.NotNull;
import jakarta.validation.constraints.Pattern;

import org.globalart.astrum.repository.config.UniqueRepositoryName;
import org.globalart.astrum.validation.constraint.NamePatternConstants;
import org.globalart.astrum.validation.group.Create;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;
import jakarta.validation.constraints.NotEmpty;

/**
 * @since 3.20
 */
public abstract class AbstractRepositoryApiRequest
    implements RepositoryApiRequest
{
  @Schema(description = "A unique identifier for this repository", example = "internal",
      requiredMode = Schema.RequiredMode.REQUIRED)
  @Pattern(regexp = NamePatternConstants.REGEX, message = NamePatternConstants.MESSAGE)
  @NotEmpty
  @UniqueRepositoryName(groups = Create.class)
  protected String name;

  @Schema(description = "Component format used by this repository", hidden = true)
  @NotEmpty
  protected String format;

  @Schema(description = "Controls if deployments of and updates to artifacts are allowed", hidden = true)
  @NotEmpty
  protected String type;

  @Schema(description = "Whether this repository accepts incoming requests", example = "true",
      requiredMode = Schema.RequiredMode.REQUIRED)
  @NotNull
  protected Boolean online;

  @JsonCreator
  public AbstractRepositoryApiRequest(
      @JsonProperty("name") final String name,
      @JsonProperty("format") final String format,
      @JsonProperty("type") final String type,
      @JsonProperty("online") final Boolean online)
  {
    this.name = name;
    this.format = format;
    this.type = type;
    this.online = online;
  }

  @Override
  public String getName() {
    return name;
  }

  @Override
  public String getFormat() {
    return format;
  }

  @Override
  public String getType() {
    return type;
  }

  @Override
  public Boolean getOnline() {
    return online;
  }
}
