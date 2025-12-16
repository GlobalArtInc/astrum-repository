
package org.globalart.astrum.security;

import java.io.Serializable;

import static com.google.common.base.Preconditions.checkNotNull;

public class RealmCaseMapping
    implements Serializable
{
  private final String realmName;

  private final boolean isCaseSensitive;

  public RealmCaseMapping(final String realmName, final boolean isCaseSensitive) {
    this.realmName = checkNotNull(realmName);
    this.isCaseSensitive = isCaseSensitive;
  }

  public String getRealmName() {
    return realmName;
  }

  public boolean isCaseSensitive() {
    return isCaseSensitive;
  }
}
