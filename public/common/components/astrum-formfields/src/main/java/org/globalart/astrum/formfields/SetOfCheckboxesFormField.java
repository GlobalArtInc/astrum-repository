
package org.globalart.astrum.formfields;

/**
 * Set of checkboxes field.
 */
public class SetOfCheckboxesFormField
    extends AbstractFormField<Boolean>
{
  public SetOfCheckboxesFormField(String id, String label, String helpText, boolean required) {
    super(id, label, helpText, required);
  }

  public String getType() {
    return "setOfCheckboxes";
  }
}
