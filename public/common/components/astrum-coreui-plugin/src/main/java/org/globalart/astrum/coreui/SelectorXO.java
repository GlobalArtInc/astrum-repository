
package org.globalart.astrum.coreui;

import java.util.List;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.Pattern;

import org.globalart.astrum.selector.UniqueSelectorName;
import org.globalart.astrum.validation.constraint.NamePatternConstants;
import org.globalart.astrum.validation.group.Create;
import org.globalart.astrum.validation.group.Update;

/**
 * Selector exchange object.
 *
 * @since 3.0
 */
public class SelectorXO
{
  @NotBlank(groups = Update.class)
  private String id;

  @Pattern(regexp = NamePatternConstants.REGEX, message = NamePatternConstants.MESSAGE)
  @NotBlank(groups = Create.class)
  @UniqueSelectorName(groups = Create.class)
  private String name;

  @NotBlank(groups = Create.class)
  private String type;

  private String description;

  @NotBlank
  private String expression;

  private List<String> usedBy;

  private int usedByCount;

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

  public String getType() {
    return type;
  }

  public void setType(String type) {
    this.type = type;
  }

  public String getDescription() {
    return description;
  }

  public void setDescription(String description) {
    this.description = description;
  }

  public String getExpression() {
    return expression;
  }

  public void setExpression(String expression) {
    this.expression = expression;
  }

  public List<String> getUsedBy() {
    return usedBy;
  }

  public void setUsedBy(List<String> usedBy) {
    this.usedBy = usedBy;
  }

  public int getUsedByCount() {
    return usedByCount;
  }

  public void setUsedByCount(int usedByCount) {
    this.usedByCount = usedByCount;
  }

  @Override
  public String toString() {
    return "SelectorXO{" +
        "id='" + id + '\'' +
        ", name='" + name + '\'' +
        ", type='" + type + '\'' +
        ", description='" + description + '\'' +
        ", expression='" + expression + '\'' +
        ", usedBy=" + usedBy +
        ", usedByCount=" + usedByCount +
        '}';
  }
}
