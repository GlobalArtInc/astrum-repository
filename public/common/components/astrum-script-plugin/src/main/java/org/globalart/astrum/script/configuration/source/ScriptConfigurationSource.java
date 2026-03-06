
package org.globalart.astrum.script.configuration.source;

import java.util.List;
import java.util.stream.Stream;

import org.globalart.astrum.configuration.model.ApplyStatusXO;
import org.globalart.astrum.configuration.model.ConfigurationXO;
import org.globalart.astrum.configuration.model.InstanceConfigurationXO;
import org.globalart.astrum.configuration.source.AbstractConfigurationSource;
import org.globalart.astrum.script.ScriptManager;
import org.globalart.astrum.script.configuration.model.ScriptConfigurationListXO;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static java.util.Collections.emptyList;
import static java.util.stream.StreamSupport.stream;

@Component
public class ScriptConfigurationSource
    extends AbstractConfigurationSource
{
  private static final Logger LOG = LoggerFactory.getLogger(ScriptConfigurationSource.class);

  private final ScriptManager scriptManager;

  @Autowired
  public ScriptConfigurationSource(final ScriptManager scriptManager) {
    this.scriptManager = scriptManager;
  }

  @Override
  protected List<ApplyStatusXO> doApplyToInstance(final InstanceConfigurationXO instanceConfigurationXO) {
    ScriptConfigurationListXO listXO =
        (ScriptConfigurationListXO) instanceConfigurationXO.getConfigurationXO(ScriptConfigurationListXO.TYPE_ID);
    if (listXO != null) {
      return listXO.getScriptConfigurationXOs().stream().flatMap(scriptConfigurationXO -> {
        try {
          if (scriptManager.get(scriptConfigurationXO.getName()) != null) {
            LOG.warn(
                "configuration already exists {}",
                scriptConfigurationXO.getName());
            return Stream.of(getNotAppliedStatus(": " + scriptConfigurationXO.getName() + " already exists"));
          }
          scriptManager.create(
              scriptConfigurationXO.getName(),
              scriptConfigurationXO.getContent(),
              scriptConfigurationXO.getType());
          return Stream.of(getAppliedStatus(": " + scriptConfigurationXO.getName()));
        }
        catch (Exception e) {
          LOG.error("Failed to create configuration: {}", scriptConfigurationXO.getName(), e);
          return Stream.of(getFailedStatus(": " + scriptConfigurationXO.getName() + " " + e.getMessage()));
        }
      }).toList();
    }
    return emptyList();
  }

  @Override
  public void applyToExport(final InstanceConfigurationXO instanceConfigurationXO) {
    instanceConfigurationXO.addConfigurationXO(ScriptConfigurationListXO.from(stream(
        scriptManager
            .browse()
            .spliterator(),
        false).toList()));
  }

  @Override
  public String getConfigurationTypeId() {
    return ScriptConfigurationListXO.TYPE_ID;
  }

  @Override
  public Class<? extends ConfigurationXO> getConfigurationXOType() {
    return ScriptConfigurationListXO.class;
  }
}
