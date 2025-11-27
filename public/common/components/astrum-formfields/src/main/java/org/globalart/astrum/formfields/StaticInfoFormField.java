
package org.globalart.astrum.formfields;

/**
 * Static form element with label and description
 */
public class StaticInfoFormField
    extends AbstractFormField<String>
{
  public StaticInfoFormField(final String id, final String label, final String description) {
    super(id, label, description, false);
  }

  @Override
  public String getType() {
    return "staticInfo";
  }
}
