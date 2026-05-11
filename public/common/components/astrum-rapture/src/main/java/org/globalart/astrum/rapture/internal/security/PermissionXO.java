
package org.globalart.astrum.rapture.internal.security;

/**
 * Permission exchange object.
 *
 * @since 3.0
 */
public class PermissionXO
{
  private String id;

  public String getId() {
    return id;
  }

  public void setId(String id) {
    this.id = id;
  }

  @Override
  public String toString() {
    return "PermissionXO(" +
        "id:'" + id + '\'' +
        ')';
  }
}
