
package org.globalart.astrum.security.realm;

/**
 * Model of a security realm.
 *
 * @since 3.20
 */
public class SecurityRealm
{
  private String id;

  private String name;

  public SecurityRealm(final String id, final String name) {
    this.id = id;
    this.name = name;
  }

  public String getId() {
    return id;
  }

  public String getName() {
    return name;
  }

  public void setId(final String id) {
    this.id = id;
  }

  public void setName(final String name) {
    this.name = name;
  }

  @Override
  public String toString() {
    return "id=" + id + " name=" + name;
  }
}
