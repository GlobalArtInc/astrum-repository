
package org.globalart.astrum.script.configuration.model;

import java.util.Objects;

import org.globalart.astrum.script.Script;
import org.globalart.astrum.script.plugin.internal.ScriptData;

public class ScriptConfigurationXO
{
  private String content;

  private String name;

  private String type;

  public String getContent() {
    return content;
  }

  public void setContent(final String content) {
    this.content = content;
  }

  public String getName() {
    return name;
  }

  public void setName(final String name) {
    this.name = name;
  }

  public String getType() {
    return type;
  }

  public void setType(final String type) {
    this.type = type;
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ScriptConfigurationXO that = (ScriptConfigurationXO) o;
    return Objects.equals(getContent(), that.getContent()) &&
        Objects.equals(getName(), that.getName()) && Objects.equals(getType(), that.getType());
  }

  @Override
  public int hashCode() {
    return Objects.hash(getContent(), getName(), getType());
  }

  public Script toConfiguration() {
    Script script = new ScriptData();

    script.setName(getName());
    script.setContent(getContent());
    script.setType(getType());

    return script;
  }

  public static ScriptConfigurationXO from(final Script configuration) {
    ScriptConfigurationXO xo = new ScriptConfigurationXO();

    xo.setContent(configuration.getContent());
    xo.setName(configuration.getName());
    xo.setType(configuration.getType());

    return xo;
  }
}
