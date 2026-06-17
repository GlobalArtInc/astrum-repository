
package org.globalart.astrum.coreui.internal.maliciousrisk;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.globalart.astrum.common.app.FeatureFlags.MALWARE_RISK_ENABLED;
import static org.globalart.astrum.common.app.FeatureFlags.MALWARE_RISK_ON_DISK_NONADMIN_OVERRIDE_ENABLED;

public class MaliciousRiskOnDiskStateContributorTest
{
  @Test
  public void featureFlagRiskOnDiskShouldBeEnabled() {
    MaliciousRiskOnDiskStateContributor underTest =
        new MaliciousRiskOnDiskStateContributor(true, false);
    assertThat(underTest.getState().get(MALWARE_RISK_ENABLED), is(true));
    assertThat(underTest.getState().get(MALWARE_RISK_ON_DISK_NONADMIN_OVERRIDE_ENABLED), is(false));
  }

  @Test
  public void featureFlagRiskOnDiskShouldBeDisabled() {
    MaliciousRiskOnDiskStateContributor underTest =
        new MaliciousRiskOnDiskStateContributor(false, true);
    assertThat(underTest.getState().get(MALWARE_RISK_ENABLED), is(false));
    assertThat(underTest.getState().get(MALWARE_RISK_ON_DISK_NONADMIN_OVERRIDE_ENABLED), is(true));
  }
}
