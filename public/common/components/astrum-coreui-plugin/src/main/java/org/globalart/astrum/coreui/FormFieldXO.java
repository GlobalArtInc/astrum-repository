
package org.globalart.astrum.coreui;

import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import javax.annotation.Nullable;

import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.formfields.NumberTextFormField;
import org.globalart.astrum.formfields.Selectable;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * {@link FormField} exchange object.
 *
 * @since 3.0
 */
public class FormFieldXO
{
  private String id;

  private String type;

  private String label;

  private String helpText;

  private Boolean required;

  private Boolean disabled;

  private Boolean readOnly;

  @Nullable
  private String regexValidation;

  @Nullable
  private String initialValue;

  /**
   * @since 3.1
   */
  private Map<String, Object> attributes;

  /**
   * @see NumberTextFormField
   */
  @Nullable
  private String minValue;

  /**
   * @see NumberTextFormField
   */
  @Nullable
  private String maxValue;

  /**
   * @see Selectable
   */
  @Nullable
  private String storeApi;

  /**
   * @see Selectable
   */
  @Nullable
  private Map<String, String> storeFilters;

  /**
   * @see Selectable
   */
  @Nullable
  private String idMapping;

  /**
   * @see Selectable
   */
  @Nullable
  private String nameMapping;

  private boolean allowAutocomplete;

  public String getId() {
    return id;
  }

  public void setId(String id) {
    this.id = id;
  }

  public String getType() {
    return type;
  }

  public void setType(String type) {
    this.type = type;
  }

  public String getLabel() {
    return label;
  }

  public void setLabel(String label) {
    this.label = label;
  }

  public String getHelpText() {
    return helpText;
  }

  public void setHelpText(String helpText) {
    this.helpText = helpText;
  }

  public Boolean getRequired() {
    return required;
  }

  public void setRequired(Boolean required) {
    this.required = required;
  }

  public Boolean getDisabled() {
    return disabled;
  }

  public void setDisabled(Boolean disabled) {
    this.disabled = disabled;
  }

  public Boolean getReadOnly() {
    return readOnly;
  }

  public void setReadOnly(Boolean readOnly) {
    this.readOnly = readOnly;
  }

  public String getRegexValidation() {
    return regexValidation;
  }

  public void setRegexValidation(String regexValidation) {
    this.regexValidation = regexValidation;
  }

  public String getInitialValue() {
    return initialValue;
  }

  public void setInitialValue(String initialValue) {
    this.initialValue = initialValue;
  }

  public Map<String, Object> getAttributes() {
    return attributes;
  }

  public void setAttributes(Map<String, Object> attributes) {
    this.attributes = attributes;
  }

  public String getMinValue() {
    return minValue;
  }

  public void setMinValue(String minValue) {
    this.minValue = minValue;
  }

  public String getMaxValue() {
    return maxValue;
  }

  public void setMaxValue(String maxValue) {
    this.maxValue = maxValue;
  }

  public String getStoreApi() {
    return storeApi;
  }

  public void setStoreApi(String storeApi) {
    this.storeApi = storeApi;
  }

  public Map<String, String> getStoreFilters() {
    return storeFilters;
  }

  public void setStoreFilters(Map<String, String> storeFilters) {
    this.storeFilters = storeFilters;
  }

  public String getIdMapping() {
    return idMapping;
  }

  public void setIdMapping(String idMapping) {
    this.idMapping = idMapping;
  }

  public String getNameMapping() {
    return nameMapping;
  }

  public void setNameMapping(String nameMapping) {
    this.nameMapping = nameMapping;
  }

  public boolean isAllowAutocomplete() {
    return allowAutocomplete;
  }

  public void setAllowAutocomplete(boolean allowAutocomplete) {
    this.allowAutocomplete = allowAutocomplete;
  }

  /**
   * Create transfer object from field source.
   *
   * @since 3.1
   */
  public static FormFieldXO create(final FormField<?> source) {
    checkNotNull(source);

    FormFieldXO result = new FormFieldXO();
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

    // FIXME: transfer objects really should not change the field names; adds unneeded confusion and complexity
    if (source instanceof NumberTextFormField ntf) {
      result.setMinValue(Optional.ofNullable(ntf.getMinimumValue()).map(Object::toString).orElse(null));
      result.setMaxValue(Optional.ofNullable(ntf.getMaximumValue()).map(Object::toString).orElse(null));
    }

    if (source instanceof Selectable selectable) {
      result.setStoreApi(selectable.getStoreApi());
      result.setStoreFilters(selectable.getStoreFilters());
      result.setAllowAutocomplete(source.getAllowAutocomplete());
      result.setIdMapping(selectable.getIdMapping());
      result.setNameMapping(selectable.getNameMapping());
    }

    return result;
  }

  @Override
  public String toString() {
    return "FormFieldXO{" +
        "id='" + id + '\'' +
        ", type='" + type + '\'' +
        ", label='" + label + '\'' +
        ", helpText='" + helpText + '\'' +
        ", required=" + required +
        ", disabled=" + disabled +
        ", readOnly=" + readOnly +
        ", regexValidation='" + regexValidation + '\'' +
        ", initialValue='" + initialValue + '\'' +
        ", attributes=" + attributes +
        ", minValue='" + minValue + '\'' +
        ", maxValue='" + maxValue + '\'' +
        ", storeApi='" + storeApi + '\'' +
        ", storeFilters=" + storeFilters +
        ", idMapping='" + idMapping + '\'' +
        ", nameMapping='" + nameMapping + '\'' +
        ", allowAutocomplete=" + allowAutocomplete +
        '}';
  }
}
