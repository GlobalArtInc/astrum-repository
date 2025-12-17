
package org.globalart.astrum.security.config;

import java.io.Serializable;

/**
 * Persistent user.
 *
 * @since 3.0
 */
public interface CUser // NOSONAR
    extends Serializable, Cloneable
{
  String STATUS_DISABLED = "disabled";

  String STATUS_ACTIVE = "active";

  String STATUS_CHANGE_PASSWORD = "changepassword";

  String getEmail();

  String getFirstName();

  String getId();

  String getLastName();

  String getPassword();

  String getStatus();

  void setEmail(String email);

  void setFirstName(String firstName);

  void setId(String id);

  void setLastName(String lastName);

  void setPassword(String password);

  void setStatus(String status);

  int getVersion();

  void setVersion(final int version);

  boolean isActive();

  CUser clone();
}
