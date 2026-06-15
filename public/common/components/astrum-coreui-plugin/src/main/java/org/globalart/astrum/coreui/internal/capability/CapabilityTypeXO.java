
package org.globalart.astrum.coreui.internal.capability;

import java.util.List;
import jakarta.validation.constraints.NotEmpty;

import org.globalart.astrum.coreui.FormFieldXO;

public class CapabilityTypeXO
{
  @NotEmpty
  private String id;

  @NotEmpty
  private String name;

  private String about;

  private List<FormFieldXO> formFields;

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

  public String getAbout() {
    return about;
  }

  public void setAbout(final String about) {
    this.about = about;
  }

  public List<FormFieldXO> getFormFields() {
    return formFields;
  }

  public void setFormFields(final List<FormFieldXO> formFields) {
    this.formFields = formFields;
  }

  @Override
  public String toString() {
    return "CapabilityTypeXO(" +
        "id:" + id +
        ", name:" + name +
        ", about:" + about +
        ", formFields:" + formFields +
        ")";
  }
}
