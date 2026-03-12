
package org.globalart.astrum.upgrade.cycle;

import org.globalart.astrum.common.upgrade.DependsOn;
import org.globalart.astrum.common.upgrade.Upgrade;
import org.globalart.astrum.common.upgrade.Upgrades;

@Upgrades(model = "foo", from = "1.0", to = "1.1")
@DependsOn(model = "wibble", version = "1.1")
public class UpgradeFoo_1_1
    implements Upgrade
{
  @Override
  public void apply() {
    // no-op
  }
}
