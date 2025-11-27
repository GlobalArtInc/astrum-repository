
package org.globalart.astrum.formfields;

/**
 * URL field.
 *
 * @since 3.2
 */
public class UrlFormField
    extends StringTextFormField
{
  public UrlFormField(String id, String label, String helpText, boolean required, String regexValidation) {
    super(id, label, helpText, required, regexValidation);
  }

  public UrlFormField(String id, String label, String helpText, boolean required) {
    super(id, label, helpText, required);
  }

  public UrlFormField(String id) {
    super(id);
  }

  public String getType() {
    return "url";
  }

}
