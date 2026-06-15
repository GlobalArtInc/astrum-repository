
package org.globalart.astrum.coreui;

import java.util.List;
import java.util.Objects;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.Pattern;

import org.globalart.astrum.repository.routing.RoutingMode;
import org.globalart.astrum.validation.constraint.NamePatternConstants;
import org.globalart.astrum.validation.group.Create;

/**
 * Routing Rule transfer object for internal REST API.
 *
 * @since 3.16
 */
public class RoutingRuleXO
{
  private String id;

  @Pattern(regexp = NamePatternConstants.REGEX, message = NamePatternConstants.MESSAGE)
  @NotBlank(groups = Create.class)
  private String name;

  private String description;

  @NotBlank(groups = Create.class)
  private RoutingMode mode;

  @NotBlank
  private List<String> matchers;

  private int assignedRepositoryCount;

  private List<String> assignedRepositoryNames;

  public String getId() {
    return id;
  }

  public void setId(String id) {
    this.id = id;
  }

  public String getName() {
    return name;
  }

  public void setName(String name) {
    this.name = name;
  }

  public String getDescription() {
    return description;
  }

  public void setDescription(String description) {
    this.description = description;
  }

  public RoutingMode getMode() {
    return mode;
  }

  public void setMode(RoutingMode mode) {
    this.mode = mode;
  }

  public List<String> getMatchers() {
    return matchers;
  }

  public void setMatchers(List<String> matchers) {
    this.matchers = matchers;
  }

  public int getAssignedRepositoryCount() {
    return assignedRepositoryCount;
  }

  public void setAssignedRepositoryCount(int assignedRepositoryCount) {
    this.assignedRepositoryCount = assignedRepositoryCount;
  }

  public List<String> getAssignedRepositoryNames() {
    return assignedRepositoryNames;
  }

  public void setAssignedRepositoryNames(List<String> assignedRepositoryNames) {
    this.assignedRepositoryNames = assignedRepositoryNames;
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    RoutingRuleXO that = (RoutingRuleXO) o;
    return Objects.equals(name, that.name);
  }

  @Override
  public int hashCode() {
    return Objects.hash(name);
  }

  @Override
  public String toString() {
    return "RoutingRuleXO{" +
        "id='" + id + '\'' +
        ", name='" + name + '\'' +
        ", description='" + description + '\'' +
        ", mode=" + mode +
        ", matchers=" + matchers +
        ", assignedRepositoryCount=" + assignedRepositoryCount +
        ", assignedRepositoryNames=" + assignedRepositoryNames +
        '}';
  }
}
