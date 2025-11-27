
package org.globalart.astrum.formfields;

/**
 * A {@link StringTextFormField} that masks the input.
 *
 * @since 2.7
 */
public class PasswordFormField
    extends StringTextFormField
    implements Encrypted
{
  public PasswordFormField(String id, String label, String helpText, boolean required, String regexValidation) {
    super(id, label, helpText, required, regexValidation);
  }

  public PasswordFormField(String id, String label, String helpText, boolean required) {
    super(id, label, helpText, required);
  }

  public PasswordFormField(String id) {
    super(id);
  }

  public String getType() {
    return "password";
  }

  public PasswordFormField withInitialValue(final String initialValue) {
    super.withInitialValue(initialValue);
    return this;
  }

}
