
package org.globalart.astrum.coreui.internal.roles;

/**
 * Response object for role sources.
 */
public class RoleSourceUIResponse
{
  private String id;

  private String name;

  public RoleSourceUIResponse() {
    // for deserialization
  }

  public RoleSourceUIResponse(final String id, final String name) {
    this.id = id;
    this.name = name;
  }

  public String getId() {
    return id;
  }

  public void setId(final String id) {
    this.id = id;
  }

  public String getName() {
    return name;
  }

  public void setName(final String name) {
    this.name = name;
  }
}
