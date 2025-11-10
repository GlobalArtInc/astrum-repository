
package org.globalart.astrum.configuration.source;

import java.util.List;

import org.globalart.astrum.configuration.model.ApplyStatusXO;
import org.globalart.astrum.configuration.model.ConfigurationXO;
import org.globalart.astrum.configuration.model.InstanceConfigurationXO;

public interface ConfigurationSource
{
  List<ApplyStatusXO> applyToInstance(final InstanceConfigurationXO instanceConfigurationXO);

  void applyToExport(final InstanceConfigurationXO instanceConfigurationXO);

  String getConfigurationTypeId();

  Class<? extends ConfigurationXO> getConfigurationXOType();
}
