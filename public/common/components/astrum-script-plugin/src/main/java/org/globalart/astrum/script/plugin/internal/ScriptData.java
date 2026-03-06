
package org.globalart.astrum.script.plugin.internal;

import java.util.Objects;

import org.globalart.astrum.common.entity.HasName;
import org.globalart.astrum.script.Script;
import org.globalart.astrum.script.ScriptManager;

/**
 * {@link Script} data.
 *
 * @since 3.21
 */
public class ScriptData
    implements HasName, Script
{
  private String name;

  private String content;

  private String type = ScriptManager.DEFAULT_TYPE;

  public ScriptData() {
  }

  public ScriptData(String name, String content, String type) {
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
    return "ScriptData{" +
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
    ScriptData that = (ScriptData) o;
    return Objects.equals(name, that.name) &&
        Objects.equals(content, that.content) &&
        Objects.equals(type, that.type);
  }

  @Override
  public int hashCode() {
    return Objects.hash(name, content, type);
  }
}
