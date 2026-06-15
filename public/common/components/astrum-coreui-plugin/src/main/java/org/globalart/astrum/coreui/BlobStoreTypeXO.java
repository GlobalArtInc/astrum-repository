
package org.globalart.astrum.coreui;

import java.util.List;
import jakarta.validation.constraints.NotBlank;

/**
 * BlobStore Type exchange object.
 *
 * @since 3.6
 */
public class BlobStoreTypeXO
{
  @NotBlank
  private String id;

  @NotBlank
  private String name;

  private List<FormFieldXO> formFields;

  private String customFormName;

  private boolean isModifiable;

  private boolean isEnabled;

  private boolean isConnectionTestable;

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

  public String getCustomFormName() {
    return customFormName;
  }

  public void setCustomFormName(String customFormName) {
    this.customFormName = customFormName;
  }

  public boolean isModifiable() {
    return isModifiable;
  }

  public void setIsModifiable(boolean isModifiable) {
    this.isModifiable = isModifiable;
  }

  public boolean isEnabled() {
    return isEnabled;
  }

  public void setIsEnabled(boolean isEnabled) {
    this.isEnabled = isEnabled;
  }

  public boolean isConnectionTestable() {
    return isConnectionTestable;
  }

  public void setConnectionTestable(boolean isConnectionTestable) {
    this.isConnectionTestable = isConnectionTestable;
  }

  @Override
  public String toString() {
    return "BlobStoreTypeXO{" +
        "id='" + id + '\'' +
        ", name='" + name + '\'' +
        ", formFields=" + formFields +
        ", customFormName='" + customFormName + '\'' +
        ", isModifiable=" + isModifiable +
        ", isEnabled=" + isEnabled +
        ", isConnectionTestable=" + isConnectionTestable +
        '}';
  }
}
