
package org.globalart.astrum.coreui.internal.maliciousrisk;

import java.util.Map;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.rapture.StateContributor;

import com.google.common.collect.ImmutableMap;
import org.springframework.beans.factory.annotation.Value;

import static org.globalart.astrum.common.app.FeatureFlags.MALWARE_RISK_ENABLED;
import static org.globalart.astrum.common.app.FeatureFlags.MALWARE_RISK_ENABLED_NAMED_VALUE;
import static org.globalart.astrum.common.app.FeatureFlags.MALWARE_RISK_ON_DISK_NONADMIN_OVERRIDE_ENABLED;
import static org.globalart.astrum.common.app.FeatureFlags.MALWARE_RISK_ON_DISK_NONADMIN_OVERRIDE_ENABLED_NAMED_VALUE;
import org.springframework.stereotype.Component;

@Component
public class MaliciousRiskOnDiskStateContributor
    implements StateContributor
{
  private final Map<String, Object> state;

  @Autowired
  public MaliciousRiskOnDiskStateContributor(
      @Value(MALWARE_RISK_ENABLED_NAMED_VALUE) final boolean maliciousRiskEnabled,
      @Value(MALWARE_RISK_ON_DISK_NONADMIN_OVERRIDE_ENABLED_NAMED_VALUE) final boolean maliciousRiskOnDiskNoneAdminOverrideEnabled)
  {
    this.state = ImmutableMap.of(
        MALWARE_RISK_ENABLED, maliciousRiskEnabled,
        MALWARE_RISK_ON_DISK_NONADMIN_OVERRIDE_ENABLED, maliciousRiskOnDiskNoneAdminOverrideEnabled);
  }

  @Override
  public Map<String, Object> getState() {
    return state;
  }
}
