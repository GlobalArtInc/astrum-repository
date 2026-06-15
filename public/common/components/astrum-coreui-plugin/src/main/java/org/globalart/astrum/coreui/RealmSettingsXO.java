
package org.globalart.astrum.coreui;

import java.util.List;

/**
 * Realm Security Settings exchange object.
 *
 * @since 3.0
 */
public class RealmSettingsXO
{
  private List<String> realms;

  public List<String> getRealms() {
    return realms;
  }

  public void setRealms(List<String> realms) {
    this.realms = realms;
  }

  @Override
  public String toString() {
    return "RealmSettingsXO{" +
        "realms=" + realms +
        '}';
  }
}
