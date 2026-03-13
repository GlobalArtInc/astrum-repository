
package org.globalart.astrum.upgrade.example;

import org.globalart.astrum.common.upgrade.Upgrade;

import static org.mockito.Mockito.mock;

public abstract class UpgradeMock
    implements Upgrade
{
  public final Upgrade mock = mock(Upgrade.class);

  @Override
  public void apply() throws Exception {
    mock.apply();
  }
}
