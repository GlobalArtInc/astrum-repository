
package org.globalart.astrum.security;

import org.apache.shiro.authc.SimpleAuthenticationInfo;

public class AstrumSimpleAuthenticationInfo
    extends SimpleAuthenticationInfo
{
  public AstrumSimpleAuthenticationInfo(
      final String principal,
      final char[] credentials,
      final RealmCaseMapping realmCaseMapping)
  {
    this.principals = new AstrumSimplePrincipalCollection(principal, realmCaseMapping);
    this.credentials = credentials;
  }
}
