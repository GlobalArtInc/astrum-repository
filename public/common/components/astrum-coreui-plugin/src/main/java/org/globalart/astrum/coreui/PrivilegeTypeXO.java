
package org.globalart.astrum.coreui;

import java.util.List;
import jakarta.validation.constraints.NotEmpty;

/**
 * Privilege Type exchange object.
 *
 * @since 3.0
 */
public class PrivilegeTypeXO
{
  @NotEmpty
  private String id;

  @NotEmpty
  private String name;

  private List<FormFieldXO> formFields;

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

  public List<FormFieldXO> getFormFields() {
    return formFields;
  }

  public void setFormFields(List<FormFieldXO> formFields) {
    this.formFields = formFields;
  }

  @Override
  public String toString() {
    return "PrivilegeTypeXO{" +
        "id='" + id + '\'' +
        ", name='" + name + '\'' +
        ", formFields=" + formFields +
        '}';
  }
}
