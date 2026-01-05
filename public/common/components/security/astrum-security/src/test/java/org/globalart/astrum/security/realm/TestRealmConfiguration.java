
package org.globalart.astrum.security.realm;

import java.util.List;

public class TestRealmConfiguration
    implements RealmConfiguration
{
  private List<String> realmNames;

  @Override
  public List<String> getRealmNames() {
    return realmNames;
  }

  @Override
  public void setRealmNames(final List<String> realmNames) {
    this.realmNames = realmNames;
  }

  @Override
  public RealmConfiguration copy() {
    return this;
  }
}
