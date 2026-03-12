
package org.globalart.astrum.upgrade.duplicate;

import org.globalart.astrum.common.upgrade.Upgrades;
import org.globalart.astrum.upgrade.example.UpgradeMock;

@Upgrades(model = "foo", from = "1.2", to = "1.3")
public class UpgradeFoo_1_3
    extends UpgradeMock
{
}
