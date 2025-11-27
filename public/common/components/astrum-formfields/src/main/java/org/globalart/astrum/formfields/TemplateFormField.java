
package org.globalart.astrum.formfields;

public class TemplateFormField<T>
    extends AbstractFormField<T>
{
  public TemplateFormField(
      final String id,
      final boolean required,
      final T initialValue)
  {
    super(id, null, null, required, null, initialValue);
  }

  @Override
  public String getType() {
    return "templateOnly";
  }
}
