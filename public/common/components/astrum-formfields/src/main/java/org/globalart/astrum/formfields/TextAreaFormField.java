
package org.globalart.astrum.formfields;

/**
 * A text area form field allowing input for large amount of texts in a multi-line fashion.
 *
 * @since 2.0
 */
public class TextAreaFormField
    extends AbstractFormField<String>
{
  public TextAreaFormField(
      String id,
      String label,
      String helpText,
      boolean required,
      String regexValidation,
      boolean readOnly)
  {
    super(id, label, helpText, required, regexValidation);
    this.setReadOnly(readOnly);
  }

  public TextAreaFormField(String id, String label, String helpText, boolean required, String regexValidation) {
    super(id, label, helpText, required, regexValidation);
  }

  public TextAreaFormField(String id, String label, String helpText, boolean required) {
    super(id, label, helpText, required);
  }

  public TextAreaFormField(String id) {
    super(id);
  }

  public String getType() {
    return "text-area";
  }

  public TextAreaFormField withInitialValue(final String initialValue) {
    setInitialValue(initialValue);
    return this;
  }
}
