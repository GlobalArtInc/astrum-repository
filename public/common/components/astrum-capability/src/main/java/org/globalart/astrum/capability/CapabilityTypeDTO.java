
package org.globalart.astrum.capability;

import java.util.List;
import java.util.Objects;

import io.swagger.v3.oas.annotations.media.Schema;
import io.swagger.v3.oas.annotations.media.Schema;

@Schema(description = "CapabilityType")
public class CapabilityTypeDTO
{
  @Schema(description = "Capability type identifier")
  private String id;

  @Schema(description = "Display name of the capability type")
  private String name;

  @Schema(description = "Description of the capability type")
  private String about;

  @Schema(description = "Form fields configuration for this capability type")
  private List<FormFieldDTO> formFields;

  protected CapabilityTypeDTO() {
    // deserialization
  }

  public CapabilityTypeDTO(
      final String id,
      final String name,
      final String about,
      final List<FormFieldDTO> formFields)
  {
    this.id = id;
    this.name = name;
    this.about = about;
    this.formFields = formFields;
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

  public String getAbout() {
    return about;
  }

  public void setAbout(final String about) {
    this.about = about;
  }

  public List<FormFieldDTO> getFormFields() {
    return formFields;
  }

  public void setFormFields(final List<FormFieldDTO> formFields) {
    this.formFields = formFields;
  }

  @Override
  public String toString() {
    return "CapabilityTypeDTO(" +
        "id:" + id +
        ", name:" + name +
        ", about:" + about +
        ", formFields:" + formFields +
        ")";
  }

  @Override
  public boolean equals(final Object o) {
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    CapabilityTypeDTO that = (CapabilityTypeDTO) o;
    return Objects.equals(id, that.id) && Objects.equals(name, that.name) &&
        Objects.equals(about, that.about) && Objects.equals(formFields, that.formFields);
  }

  @Override
  public int hashCode() {
    return Objects.hash(id, name, about, formFields);
  }
}
