
package org.globalart.astrum.security;

import java.util.Objects;

import org.apache.shiro.subject.SimplePrincipalCollection;

import static com.google.common.base.Preconditions.checkNotNull;

public class AstrumSimplePrincipalCollection
    extends SimplePrincipalCollection
{
  private final RealmCaseMapping realmCaseMapping;

  public AstrumSimplePrincipalCollection(final String principal, final RealmCaseMapping realmCaseMapping) {
    super(principal, realmCaseMapping.getRealmName());
    this.realmCaseMapping = checkNotNull(realmCaseMapping);
  }

  public String getRealmName() {
    return realmCaseMapping.getRealmName();
  }

  public boolean isRealmCaseSensitive() {
    return realmCaseMapping.isCaseSensitive();
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    if (!super.equals(o)) {
      return false;
    }
    AstrumSimplePrincipalCollection that = (AstrumSimplePrincipalCollection) o;
    return Objects.equals(realmCaseMapping, that.realmCaseMapping);
  }

  @Override
  public int hashCode() {
    return Objects.hash(super.hashCode(), realmCaseMapping);
  }
}
