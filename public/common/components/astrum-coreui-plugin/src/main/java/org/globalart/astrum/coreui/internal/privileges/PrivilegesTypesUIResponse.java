
package org.globalart.astrum.coreui.internal.privileges;

import java.util.List;
import java.util.Map;

import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.security.privilege.PrivilegeDescriptor;

public class PrivilegesTypesUIResponse
{
  private final String id;

  private final String name;

  private final List<FormField> formFields;

  public PrivilegesTypesUIResponse(final Map.Entry<String, PrivilegeDescriptor> entry) {
    PrivilegeDescriptor privilegeDescriptor = entry.getValue();

    this.id = privilegeDescriptor.getType();
    this.name = privilegeDescriptor.getName();
    this.formFields = privilegeDescriptor.getFormFields();
  }

  public String getId() {
    return id;
  }

  public String getName() {
    return name;
  }

  public List<FormField> getFormFields() {
    return formFields;
  }
}
