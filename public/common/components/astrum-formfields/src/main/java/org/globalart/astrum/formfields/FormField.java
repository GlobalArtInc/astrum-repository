
package org.globalart.astrum.formfields;

import java.util.Map;

import javax.annotation.Nullable;

/**
 * Form field.
 *
 * @param <T> The data type of the field.
 */
public interface FormField<T>
{
  /**
   * Mandatory ({@code true}) symbol.
   *
   * @see #isRequired()
   */
  boolean MANDATORY = true;

  /**
   * Optional ({@code false}) symbol.
   *
   * @see #isRequired()
   */
  boolean OPTIONAL = false;

  /**
   * Field type.
   *
   * This is a symbolic type to match up the widget implementation for the field in the UI.
   */
  String getType();

  /**
   * Field label.
   */
  String getLabel();

  /**
   * Field identifier.
   */
  String getId();

  /**
   * True if field is required.
   */
  boolean isRequired();

  /**
   * True if field is disabled.
   */
  boolean isDisabled();

  /**
   * True if field can only read.
   */
  boolean isReadOnly();

  /**
   * Help text of field.
   */
  String getHelpText();

  /**
   * Optional regular-expression to validate field.
   */
  @Nullable
  String getRegexValidation();

  /**
   * Optional initial value of the field.
   *
   * @since 2.3
   */
  @Nullable
  T getInitialValue();

  /**
   * Optional field attributes.
   *
   * Used to encode additional data to widget implementation in UI.
   *
   * Care must be used to ensure that values are transferable, and likely should remain simple values,
   * collections of simple values or simple transfer objects.
   *
   * @since 3.1
   */
  Map<String, Object> getAttributes();

  default boolean getAllowAutocomplete() {
    return false;
  }
}
