
package org.globalart.astrum.coreui;

import java.util.Objects;
import jakarta.validation.constraints.NotBlank;

/**
 * Reference exchange object.
 *
 * @since 3.0
 * @deprecated in favor of same class in the astrum-extdirect-api-common module
 */
@Deprecated
public class ReferenceXO
{
  @NotBlank
  private String id;

  @NotBlank
  private String name;

  public ReferenceXO() {
    // empty
  }

  public ReferenceXO(final String id, final String name) {
    this.id = id;
    this.name = name;
  }

  public String getId() {
    return id;
  }

  public void setId(String id) {
    this.id = id;
  }

  public String getName() {
    return name;
  }

  public void setName(String name) {
    this.name = name;
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ReferenceXO other = (ReferenceXO) o;
    return Objects.equals(id, other.id) && Objects.equals(name, other.name);
  }

  @Override
  public int hashCode() {
    return Objects.hash(id, name);
  }

  @Override
  public String toString() {
    return "ReferenceXO{" +
        "id='" + id + '\'' +
        ", name='" + name + '\'' +
        '}';
  }
}
