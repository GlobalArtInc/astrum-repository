
package org.globalart.astrum.coreui;

import java.util.Map;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotEmpty;
import jakarta.validation.constraints.NotNull;
import jakarta.validation.constraints.Pattern;

import org.globalart.astrum.repository.config.UniqueRepositoryName;
import org.globalart.astrum.validation.constraint.NamePatternConstants;
import org.globalart.astrum.validation.group.Create;

/**
 * Repository exchange object.
 *
 * @since 3.0
 */
@DockerRepositoryNameConstraint(groups = Create.class)
public class RepositoryXO
{
  @Pattern(regexp = NamePatternConstants.REGEX, message = NamePatternConstants.MESSAGE)
  @NotEmpty
  @UniqueRepositoryName(groups = Create.class)
  private String name;

  private String type;

  private String format;

  private Long size;

  @NotBlank(groups = Create.class)
  private String recipe;

  @NotNull
  private Boolean online;

  private String routingRuleId;

  @NotEmpty
  private Map<String, Map<String, Object>> attributes;

  private String url;

  private RepositoryStatusXO status;

  public String getName() {
    return name;
  }

  public void setName(String name) {
    this.name = name;
  }

  public String getType() {
    return type;
  }

  public void setType(String type) {
    this.type = type;
  }

  public String getFormat() {
    return format;
  }

  public void setFormat(String format) {
    this.format = format;
  }

  public Long getSize() {
    return size;
  }

  public void setSize(Long size) {
    this.size = size;
  }

  public String getRecipe() {
    return recipe;
  }

  public void setRecipe(String recipe) {
    this.recipe = recipe;
  }

  public Boolean getOnline() {
    return online;
  }

  public void setOnline(Boolean online) {
    this.online = online;
  }

  public String getRoutingRuleId() {
    return routingRuleId;
  }

  public void setRoutingRuleId(String routingRuleId) {
    this.routingRuleId = routingRuleId;
  }

  public Map<String, Map<String, Object>> getAttributes() {
    return attributes;
  }

  public void setAttributes(Map<String, Map<String, Object>> attributes) {
    this.attributes = attributes;
  }

  public String getUrl() {
    return url;
  }

  public void setUrl(String url) {
    this.url = url;
  }

  public RepositoryStatusXO getStatus() {
    return status;
  }

  public void setStatus(RepositoryStatusXO status) {
    this.status = status;
  }

  @Override
  public String toString() {
    return "RepositoryXO{" +
        "name='" + name + '\'' +
        ", type='" + type + '\'' +
        ", format='" + format + '\'' +
        ", size=" + size +
        ", recipe='" + recipe + '\'' +
        ", online=" + online +
        ", routingRuleId='" + routingRuleId + '\'' +
        ", attributes=" + attributes +
        ", url='" + url + '\'' +
        ", status=" + status +
        '}';
  }
}
