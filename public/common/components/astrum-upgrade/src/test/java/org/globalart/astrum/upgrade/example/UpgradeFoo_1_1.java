
package org.globalart.astrum.upgrade.example;

import org.globalart.astrum.common.upgrade.Upgrades;

@Upgrades(model = "foo", from = "1.0", to = "1.1")
public class UpgradeFoo_1_1
    extends UpgradeMock
{
}
