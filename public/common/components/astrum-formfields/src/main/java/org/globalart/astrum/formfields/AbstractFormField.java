
package org.globalart.astrum.formfields;

import java.util.HashMap;
import java.util.Map;

import javax.annotation.Nullable;

/**
 * Support for {@link FormField} implementations.
 */
public abstract class AbstractFormField<T>
    implements FormField<T>
{
  private String helpText;

  private String id;

  @Nullable
  private String regexValidation;

  private boolean required;

  private boolean disabled;

  private boolean readOnly;

  private String label;

  @Nullable
  private T initialValue;

  /**
   * @since 3.1
   */
  private Map<String, Object> attributes;

  // NOTE: avoid adding anymore constructors

  public AbstractFormField(
      final String id,
      final String label,
      final String helpText,
      final boolean required,
      @Nullable final String regexValidation,
      @Nullable final T initialValue)
  {
    this(id, label, helpText, required, regexValidation);
    this.initialValue = initialValue;
  }

  public AbstractFormField(
      final String id,
      final String label,
      final String helpText,
      final boolean required,
      @Nullable final String regexValidation)
  {
    this(id, label, helpText, required);
    this.regexValidation = regexValidation;
  }

  public AbstractFormField(
      final String id,
      final String label,
      final String helpText,
      final boolean required)
  {
    this(id);
    this.label = label;
    this.helpText = helpText;
    this.required = required;
  }

  public AbstractFormField(final String id) {
    this.id = id;
  }

  public String getLabel() {
    return this.label;
  }

  public String getHelpText() {
    return this.helpText;
  }

  public String getId() {
    return this.id;
  }

  @Nullable
  public String getRegexValidation() {
    return this.regexValidation;
  }

  public boolean isRequired() {
    return this.required;
  }

  public boolean isDisabled() {
    return this.disabled;
  }

  public boolean isReadOnly() {
    return this.readOnly;
  }

  @Nullable
  public T getInitialValue() {
    return initialValue;
  }

  public void setHelpText(final String helpText) {
    this.helpText = helpText;
  }

  public void setId(final String id) {
    this.id = id;
  }

  public void setRegexValidation(@Nullable final String regex) {
    this.regexValidation = regex;
  }

  public void setRequired(final boolean required) {
    this.required = required;
  }

  public void setDisabled(final boolean disabled) {
    this.disabled = disabled;
  }

  public void setReadOnly(final boolean readOnly) {
    this.readOnly = readOnly;
  }

  public void setLabel(final String label) {
    this.label = label;
  }

  public void setInitialValue(@Nullable final T value) {
    this.initialValue = value;
  }

  /**
   * @since 3.1
   */
  @Override
  public Map<String, Object> getAttributes() {
    if (attributes == null) {
      attributes = new HashMap<>();
    }
    return attributes;
  }

  public AbstractFormField<T> withAttribute(String key, Object value) {
    getAttributes().put(key, value);
    return this;
  }
}
