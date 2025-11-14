
package org.globalart.astrum.internal.security.realm;

import java.util.ArrayList;
import java.util.List;

import org.globalart.astrum.security.realm.RealmConfiguration;

/**
 * {@link RealmConfiguration} data.
 *
 * @since 3.21
 */
public class RealmConfigurationData
    implements RealmConfiguration, Cloneable
{
  private List<String> realmNames = new ArrayList<>();

  @Override
  public List<String> getRealmNames() {
    return realmNames;
  }

  @Override
  public void setRealmNames(final List<String> realmNames) {
    this.realmNames = realmNames != null ? realmNames : new ArrayList<>();
  }

  @Override
  public RealmConfiguration copy() {
    try {
      return (RealmConfiguration) clone();
    }
    catch (CloneNotSupportedException e) {
      throw new RuntimeException(e);
    }
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "realmNames=" + realmNames +
        '}';
  }
}
