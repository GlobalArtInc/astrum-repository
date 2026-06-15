
package org.globalart.astrum.coreui;

import jakarta.validation.constraints.NotEmpty;

/**
 * @since 3.24
 */
public class UserAccountPasswordXO
{
  @NotEmpty
  private String authToken;

  @NotEmpty
  private String password;

  public String getAuthToken() {
    return authToken;
  }

  public void setAuthToken(String authToken) {
    this.authToken = authToken;
  }

  public String getPassword() {
    return password;
  }

  public void setPassword(String password) {
    this.password = password;
  }
}
