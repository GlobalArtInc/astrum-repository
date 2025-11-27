
package org.globalart.astrum.formfields;

import java.util.Map;

import org.globalart.astrum.common.i18n.I18N;
import org.globalart.astrum.common.i18n.MessageBundle;

/**
 * The model for a combo field allowing for selection of content selectors.
 *
 * @since 3.0
 */
public class SelectorComboFormField
    extends Combobox<String>
{

  private interface Messages
      extends MessageBundle
  {
    @DefaultMessage("Content Selector")
    String label();

    @DefaultMessage("Select the content selector.")
    String helpText();
  }

  private static final Messages messages = I18N.create(Messages.class);

  public SelectorComboFormField(
      String id,
      String label,
      String helpText,
      boolean required,
      String regexValidation)
  {
    super(id, label, helpText, required);
    withRegexValidation(regexValidation);
  }

  public SelectorComboFormField(String id, String label, String helpText, boolean required) {
    super(id, label, helpText, required);
  }

  public SelectorComboFormField(String id, boolean required) {
    super(id, messages.label(), messages.helpText(), required);
  }

  public SelectorComboFormField(String id) {
    super(id, messages.label(), messages.helpText(), false);
  }

  @Override
  public String getStoreApi() {
    return "coreui_Selector.readReferences";
  }

  @Override
  public Map<String, String> getStoreFilters() {
    return null;
  }
}
