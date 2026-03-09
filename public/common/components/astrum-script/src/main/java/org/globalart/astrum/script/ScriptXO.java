
package org.globalart.astrum.script;

import java.util.Objects;
import jakarta.validation.constraints.NotEmpty;
import jakarta.validation.constraints.Pattern;
import org.globalart.astrum.validation.constraint.NamePatternConstants;

/**
 * Script exchange object.
 *
 * @since 3.0
 */
public class ScriptXO
{

  @Pattern(regexp = NamePatternConstants.REGEX, message = NamePatternConstants.MESSAGE)
  @NotEmpty
  private String name;

  @NotEmpty
  private String content;

  @NotEmpty
  private String type;

  public ScriptXO() {
  }

  public ScriptXO(String name, String content, String type) {
    this.name = name;
    this.content = content;
    this.type = type;
  }

  public String getName() {
    return name;
  }

  public void setName(String name) {
    this.name = name;
  }

  public String getContent() {
    return content;
  }

  public void setContent(String content) {
    this.content = content;
  }

  public String getType() {
    return type;
  }

  public void setType(String type) {
    this.type = type;
  }

  @Override
  public String toString() {
    return "ScriptXO{" +
        "name='" + name + '\'' +
        ", content='" + content + '\'' +
        ", type='" + type + '\'' +
        '}';
  }

  @Override
  public boolean equals(Object o) {
    if (this == o)
      return true;
    if (o == null || getClass() != o.getClass())
      return false;
    ScriptXO scriptXO = (ScriptXO) o;
    return Objects.equals(name, scriptXO.name) &&
        Objects.equals(content, scriptXO.content) &&
        Objects.equals(type, scriptXO.type);
  }

  @Override
  public int hashCode() {
    return Objects.hash(name, content, type);
  }
}
