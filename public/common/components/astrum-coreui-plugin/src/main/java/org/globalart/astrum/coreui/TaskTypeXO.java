
package org.globalart.astrum.coreui;

import java.util.List;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotNull;

/**
 * Task type exchange object.
 *
 * @since 3.0
 */
public class TaskTypeXO
{
  @NotBlank
  private String id;

  @NotBlank
  private String name;

  @NotNull
  private Boolean exposed;

  @NotNull
  private Boolean concurrentRun;

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

  public Boolean getExposed() {
    return exposed;
  }

  public void setExposed(Boolean exposed) {
    this.exposed = exposed;
  }

  public Boolean getConcurrentRun() {
    return concurrentRun;
  }

  public void setConcurrentRun(Boolean concurrentRun) {
    this.concurrentRun = concurrentRun;
  }

  public List<FormFieldXO> getFormFields() {
    return formFields;
  }

  public void setFormFields(List<FormFieldXO> formFields) {
    this.formFields = formFields;
  }

  @Override
  public String toString() {
    return "TaskTypeXO{" +
        "id='" + id + '\'' +
        ", name='" + name + '\'' +
        ", exposed=" + exposed +
        ", concurrentRun=" + concurrentRun +
        ", formFields=" + formFields +
        '}';
  }
}
