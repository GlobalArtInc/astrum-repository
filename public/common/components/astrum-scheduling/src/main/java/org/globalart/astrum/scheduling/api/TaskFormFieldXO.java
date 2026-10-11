package org.globalart.astrum.scheduling.api;

import java.util.Map;
import java.util.Objects;
import java.util.Optional;

import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.formfields.NumberTextFormField;
import org.globalart.astrum.formfields.Selectable;

import io.swagger.v3.oas.annotations.media.Schema;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * A form field of a task type, as a client needs it to render and validate the task's settings.
 *
 * @since 3.96
 */
@Schema(description = "A setting of a task type")
public class TaskFormFieldXO
{
  @Schema(description = "The name of the task property the field edits", example = "blobstoreName")
  private String id;

  @Schema(description = "The kind of input", example = "combobox")
  private String type;

  private String label;

  private String helpText;

  private boolean required;

  private boolean disabled;

  private boolean readOnly;

  private String regexValidation;

  private String initialValue;

  private String minValue;

  private String maxValue;

  @Schema(description = "The source of the options of a selectable field", example = "coreui_Blobstore.ReadNames")
  private String storeApi;

  private Map<String, String> storeFilters;

  private String idMapping;

  private String nameMapping;

  private Boolean allowAutocomplete;

  private Map<String, Object> attributes;

  public static TaskFormFieldXO create(final FormField<?> source) {
    checkNotNull(source);
    TaskFormFieldXO result = new TaskFormFieldXO();
    result.setId(source.getId());
    result.setType(source.getType());
    result.setLabel(source.getLabel());
    result.setHelpText(source.getHelpText());
    result.setRequired(source.isRequired());
    result.setDisabled(source.isDisabled());
    result.setReadOnly(source.isReadOnly());
    result.setRegexValidation(source.getRegexValidation());
    result.setInitialValue(Optional.ofNullable(source.getInitialValue()).map(Objects::toString).orElse(null));
    result.setAttributes(source.getAttributes());
    if (source instanceof NumberTextFormField numberField) {
      result.setMinValue(Optional.ofNullable(numberField.getMinimumValue()).map(Object::toString).orElse(null));
      result.setMaxValue(Optional.ofNullable(numberField.getMaximumValue()).map(Object::toString).orElse(null));
    }
    if (source instanceof Selectable selectable) {
      result.setStoreApi(selectable.getStoreApi());
      result.setStoreFilters(selectable.getStoreFilters());
      result.setIdMapping(selectable.getIdMapping());
      result.setNameMapping(selectable.getNameMapping());
      result.setAllowAutocomplete(source.getAllowAutocomplete());
    }
    return result;
  }

  public String getId() {
    return id;
  }

  public void setId(final String id) {
    this.id = id;
  }

  public String getType() {
    return type;
  }

  public void setType(final String type) {
    this.type = type;
  }

  public String getLabel() {
    return label;
  }

  public void setLabel(final String label) {
    this.label = label;
  }

  public String getHelpText() {
    return helpText;
  }

  public void setHelpText(final String helpText) {
    this.helpText = helpText;
  }

  public boolean isRequired() {
    return required;
  }

  public void setRequired(final boolean required) {
    this.required = required;
  }

  public boolean isDisabled() {
    return disabled;
  }

  public void setDisabled(final boolean disabled) {
    this.disabled = disabled;
  }

  public boolean isReadOnly() {
    return readOnly;
  }

  public void setReadOnly(final boolean readOnly) {
    this.readOnly = readOnly;
  }

  public String getRegexValidation() {
    return regexValidation;
  }

  public void setRegexValidation(final String regexValidation) {
    this.regexValidation = regexValidation;
  }

  public String getInitialValue() {
    return initialValue;
  }

  public void setInitialValue(final String initialValue) {
    this.initialValue = initialValue;
  }

  public String getMinValue() {
    return minValue;
  }

  public void setMinValue(final String minValue) {
    this.minValue = minValue;
  }

  public String getMaxValue() {
    return maxValue;
  }

  public void setMaxValue(final String maxValue) {
    this.maxValue = maxValue;
  }

  public String getStoreApi() {
    return storeApi;
  }

  public void setStoreApi(final String storeApi) {
    this.storeApi = storeApi;
  }

  public Map<String, String> getStoreFilters() {
    return storeFilters;
  }

  public void setStoreFilters(final Map<String, String> storeFilters) {
    this.storeFilters = storeFilters;
  }

  public String getIdMapping() {
    return idMapping;
  }

  public void setIdMapping(final String idMapping) {
    this.idMapping = idMapping;
  }

  public String getNameMapping() {
    return nameMapping;
  }

  public void setNameMapping(final String nameMapping) {
    this.nameMapping = nameMapping;
  }

  public Boolean getAllowAutocomplete() {
    return allowAutocomplete;
  }

  public void setAllowAutocomplete(final Boolean allowAutocomplete) {
    this.allowAutocomplete = allowAutocomplete;
  }

  public Map<String, Object> getAttributes() {
    return attributes;
  }

  public void setAttributes(final Map<String, Object> attributes) {
    this.attributes = attributes;
  }
}
