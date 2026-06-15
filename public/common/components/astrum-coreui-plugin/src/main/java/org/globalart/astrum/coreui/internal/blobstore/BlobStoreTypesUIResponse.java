
package org.globalart.astrum.coreui.internal.blobstore;

import java.util.List;
import java.util.Map;

import org.globalart.astrum.blobstore.BlobStoreDescriptor;
import org.globalart.astrum.blobstore.SelectOption;
import org.globalart.astrum.formfields.FormField;

public class BlobStoreTypesUIResponse
{
  private final String id;

  private final String name;

  private final List<FormField> fields;

  private final String customSettingsForm;

  private final Map<String, List<SelectOption>> dropDownValues;

  public BlobStoreTypesUIResponse(final Map.Entry<String, BlobStoreDescriptor> entry) {
    BlobStoreDescriptor descriptor = entry.getValue();

    this.id = descriptor.getId();
    this.name = descriptor.getName();
    this.fields = descriptor.getFormFields();
    this.customSettingsForm = descriptor.customFormName();
    this.dropDownValues = descriptor.getDropDownValues();
  }

  public String getId() {
    return id;
  }

  public String getName() {
    return name;
  }

  public List<FormField> getFields() {
    return fields;
  }

  public String getCustomSettingsForm() {
    return customSettingsForm;
  }

  public Map<String, List<SelectOption>> getDropDownValues() {
    return dropDownValues;
  }
}
