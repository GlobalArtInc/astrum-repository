
package org.globalart.astrum.upgrade.gap;

import org.globalart.astrum.common.upgrade.Upgrade;
import org.globalart.astrum.common.upgrade.Upgrades;

@Upgrades(model = "foo", from = "1.2", to = "1.3")
public class UpgradeFoo_1_3
    implements Upgrade
{
  @Override
  public void apply() {
    // no-op
  }
}
