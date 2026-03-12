
package org.globalart.astrum.upgrade.cycle;

import org.globalart.astrum.common.upgrade.DependsOn;
import org.globalart.astrum.common.upgrade.Upgrade;
import org.globalart.astrum.common.upgrade.Upgrades;

@Upgrades(model = "bar", from = "1.0", to = "1.1")
@DependsOn(model = "foo", version = "1.1")
public class UpgradeBar_1_1
    implements Upgrade
{
  @Override
  public void apply() {
    // no-op
  }
}
