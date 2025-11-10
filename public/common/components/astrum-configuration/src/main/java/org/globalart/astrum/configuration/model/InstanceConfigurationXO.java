
package org.globalart.astrum.configuration.model;

import java.util.ArrayList;
import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class InstanceConfigurationXO
{
  private static final Logger LOG = LoggerFactory.getLogger(InstanceConfigurationXO.class);

  private List<ConfigurationXO> configurationXOs = new ArrayList<>();

  public void setConfigurationXOs(final List<ConfigurationXO> configurationXOs) {
    this.configurationXOs = configurationXOs.stream()
        .filter(Objects::nonNull)
        .collect(Collectors.toList());
  }

  public List<ConfigurationXO> getConfigurationXOs() {
    return configurationXOs;
  }

  public void addConfigurationXO(final ConfigurationXO configurationXO) {
    if (configurationXO == null) {
      LOG.debug("Ignoring null configurationXO");
    }
    else {
      configurationXOs.add(configurationXO);
    }
  }

  public ConfigurationXO getConfigurationXO(final String typeId) {
    for (ConfigurationXO configurationXO : configurationXOs) {
      if (typeId.equals(configurationXO.getConfigurationTypeId())) {
        return configurationXO;
      }
    }
    return null;
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    InstanceConfigurationXO that = (InstanceConfigurationXO) o;
    return Objects.equals(getConfigurationXOs(), that.getConfigurationXOs());
  }

  @Override
  public int hashCode() {
    return getConfigurationXOs() != null ? getConfigurationXOs().hashCode() : 0;
  }
}
