
package org.globalart.astrum.coreui.internal.capability;

import java.util.Map;

import org.globalart.astrum.rapture.StateContributor;

import com.google.common.collect.ImmutableMap;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.common.app.FeatureFlags.EXTJS_CAPABILITIES_ENABLED;
import static org.globalart.astrum.common.app.FeatureFlags.EXTJS_CAPABILITIES_NAMED_VALUE;
import static org.globalart.astrum.common.app.FeatureFlags.REACT_CAPABILITIES_ENABLED;
import static org.globalart.astrum.common.app.FeatureFlags.REACT_CAPABILITIES_NAMED_VALUE;

@Component
public class CapabilitiesPagesStateContributor
    implements StateContributor
{
  private final boolean isExtjsCapabilitiesEnabled;

  private final boolean isReactCapabilitiesEnabled;

  @Autowired
  public CapabilitiesPagesStateContributor(
      @Value(EXTJS_CAPABILITIES_NAMED_VALUE) final boolean isExtjsCapabilitiesEnabled,
      @Value(REACT_CAPABILITIES_NAMED_VALUE) final boolean isReactCapabilitiesEnabled)
  {
    this.isExtjsCapabilitiesEnabled = isExtjsCapabilitiesEnabled;
    this.isReactCapabilitiesEnabled = isReactCapabilitiesEnabled;
  }

  @Override
  public Map<String, Object> getState() {
    return ImmutableMap.of(
        EXTJS_CAPABILITIES_ENABLED, isExtjsCapabilitiesEnabled,
        REACT_CAPABILITIES_ENABLED, isReactCapabilitiesEnabled);
  }
}
