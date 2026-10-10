package org.globalart.astrum.ldap.internal;

import java.util.List;

/**
 * A user found while verifying the user and group settings.
 */
public class LdapUserXO
{
  private String username;

  private String realName;

  private String email;

  private List<String> membership;

  public String getUsername() {
    return username;
  }

  public void setUsername(final String username) {
    this.username = username;
  }

  public String getRealName() {
    return realName;
  }

  public void setRealName(final String realName) {
    this.realName = realName;
  }

  public String getEmail() {
    return email;
  }

  public void setEmail(final String email) {
    this.email = email;
  }

  public List<String> getMembership() {
    return membership;
  }

  public void setMembership(final List<String> membership) {
    this.membership = membership;
  }
}
