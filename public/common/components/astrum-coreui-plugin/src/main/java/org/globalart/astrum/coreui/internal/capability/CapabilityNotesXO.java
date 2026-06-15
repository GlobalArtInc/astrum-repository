
package org.globalart.astrum.coreui.internal.capability;

import jakarta.validation.constraints.NotEmpty;

import org.globalart.astrum.validation.group.Update;

public class CapabilityNotesXO
{
  @NotEmpty(groups = Update.class)
  private String id;

  private String notes;

  public String getId() {
    return id;
  }

  public void setId(final String id) {
    this.id = id;
  }

  public String getNotes() {
    return notes;
  }

  public void setNotes(final String notes) {
    this.notes = notes;
  }

  @Override
  public String toString() {
    return "CapabilityNotesXO(" +
        "id:" + id +
        ", notes:" + notes +
        ")";
  }
}
