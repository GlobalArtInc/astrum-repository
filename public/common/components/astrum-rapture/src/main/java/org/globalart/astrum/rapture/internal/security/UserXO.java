
package org.globalart.astrum.rapture.internal.security;

import java.util.HashSet;
import java.util.Set;

/**
 * User exchange object.
 *
 * @since 3.0
 */
public class UserXO
{
  private String id;

  private boolean authenticated;

  private boolean administrator;

  private Set<String> authenticatedRealms = new HashSet<>();

  public UserXO() {
  }

  public UserXO(String id, boolean authenticated, boolean administrator, Set<String> authenticatedRealms) {
    this.id = id;
    this.authenticated = authenticated;
    this.administrator = administrator;
    this.authenticatedRealms = authenticatedRealms;
  }

  public String getId() {
    return id;
  }

  public void setId(String id) {
    this.id = id;
  }

  public boolean isAuthenticated() {
    return authenticated;
  }

  public void setAuthenticated(boolean authenticated) {
    this.authenticated = authenticated;
  }

  public boolean isAdministrator() {
    return administrator;
  }

  public void setAdministrator(boolean administrator) {
    this.administrator = administrator;
  }

  public Set<String> getAuthenticatedRealms() {
    return authenticatedRealms;
  }

  public void setAuthenticatedRealms(Set<String> authenticatedRealms) {
    this.authenticatedRealms = authenticatedRealms;
  }

  @Override
  public String toString() {
    return "UserXO{" +
        "id='" + id + '\'' +
        ", authenticated=" + authenticated +
        ", administrator=" + administrator +
        ", authenticatedRealms=" + authenticatedRealms +
        '}';
  }
}
