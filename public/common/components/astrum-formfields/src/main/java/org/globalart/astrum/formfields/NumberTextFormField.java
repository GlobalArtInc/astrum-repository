
package org.globalart.astrum.formfields;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Number field.
 */
public class NumberTextFormField
    extends AbstractFormField<Number>
{
  private Number minimumValue;

  private Number maximumValue;

  public NumberTextFormField(String id, String label, String helpText, boolean required, String regexValidation) {
    super(id, label, helpText, required, regexValidation);
  }

  public NumberTextFormField(String id, String label, String helpText, boolean required) {
    super(id, label, helpText, required);
  }

  public NumberTextFormField(String id) {
    super(id);
  }

  public String getType() {
    return "number";
  }

  public NumberTextFormField withInitialValue(final Number initialValue) {
    setInitialValue(initialValue);
    return this;
  }

  public NumberTextFormField withMinimumValue(final Number minimumValue) {
    this.minimumValue = checkNotNull(minimumValue);
    return this;
  }

  public NumberTextFormField withMaximumValue(final Number maximumValue) {
    this.maximumValue = checkNotNull(maximumValue);
    return this;
  }

  public Number getMinimumValue() {
    return minimumValue;
  }

  public Number getMaximumValue() {
    return maximumValue;
  }
}
