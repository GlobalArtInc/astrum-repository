
package org.globalart.astrum.formfields;

/**
 * Single-line text field.
 */
public class StringTextFormField
    extends AbstractFormField<String>
{
  public StringTextFormField(String id, String label, String helpText, boolean required, String regexValidation) {
    super(id, label, helpText, required, regexValidation);
  }

  public StringTextFormField(String id, String label, String helpText, boolean required) {
    super(id, label, helpText, required);
  }

  public StringTextFormField(String id) {
    super(id);
  }

  public String getType() {
    return "string";
  }

  public StringTextFormField withInitialValue(final String initialValue) {
    setInitialValue(initialValue);
    return this;
  }

}
