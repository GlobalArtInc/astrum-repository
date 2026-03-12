
package org.globalart.astrum.upgrade.bad;

import org.globalart.astrum.common.upgrade.Upgrade;
import org.globalart.astrum.common.upgrade.Upgrades;

/**
 * Error: version doesn't change
 */
@Upgrades(model = "foo", from = "1.1", to = "1.1")
public class UpgradeFoo_1_1
    implements Upgrade
{
  @Override
  public void apply() {
    // no-op
  }
}
