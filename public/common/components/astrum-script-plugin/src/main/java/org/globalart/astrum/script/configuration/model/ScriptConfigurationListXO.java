
package org.globalart.astrum.script.configuration.model;

import java.util.ArrayList;
import java.util.List;
import java.util.Objects;

import org.globalart.astrum.configuration.model.ConfigurationXO;
import org.globalart.astrum.script.Script;

public class ScriptConfigurationListXO
    implements ConfigurationXO
{
  public static final String TYPE_ID = "script_list";

  private List<ScriptConfigurationXO> scriptConfigurationXOs = new ArrayList<>();

  public List<ScriptConfigurationXO> getScriptConfigurationXOs() {
    return scriptConfigurationXOs;
  }

  public void setScriptConfigurationXOs(final List<ScriptConfigurationXO> scriptConfigurationXOs) {
    this.scriptConfigurationXOs = scriptConfigurationXOs;
  }

  public void addScriptConfigurationXO(final ScriptConfigurationXO scriptConfigurationXO) {
    scriptConfigurationXOs.add(scriptConfigurationXO);
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ScriptConfigurationListXO that = (ScriptConfigurationListXO) o;
    return Objects.equals(getScriptConfigurationXOs(), that.getScriptConfigurationXOs());
  }

  @Override
  public int hashCode() {
    return Objects.hashCode(getScriptConfigurationXOs());
  }

  @Override
  public String getConfigurationTypeId() {
    return TYPE_ID;
  }

  public List<Script> toConfiguration() {
    List<Script> configurations = new ArrayList<>();
    scriptConfigurationXOs.forEach(scriptConfigurationXO -> {
      configurations.add(scriptConfigurationXO.toConfiguration());
    });
    return configurations;
  }

  public static ScriptConfigurationListXO from(final List<Script> configurations) {
    ScriptConfigurationListXO listXO = new ScriptConfigurationListXO();
    configurations.forEach(configuration -> {
      listXO.addScriptConfigurationXO(ScriptConfigurationXO.from(configuration));
    });
    return listXO;
  }
}
