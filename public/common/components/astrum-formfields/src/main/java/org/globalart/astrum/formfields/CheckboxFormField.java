
package org.globalart.astrum.formfields;

/**
 * Checkbox field.
 */
public class CheckboxFormField
    extends AbstractFormField<Boolean>
{
  public CheckboxFormField(String id, String label, String helpText, boolean required) {
    super(id, label, helpText, required);
  }

  public CheckboxFormField(String id) {
    super(id);
  }

  public String getType() {
    return "checkbox";
  }

  public CheckboxFormField withInitialValue(final Boolean initialValue) {
    setInitialValue(initialValue);
    return this;
  }
}
