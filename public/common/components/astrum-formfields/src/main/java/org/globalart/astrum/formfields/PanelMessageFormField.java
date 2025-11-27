

package org.globalart.astrum.formfields;

public class PanelMessageFormField
    extends AbstractFormField<String>
{
  protected static final String PANEL_TYPE_ATTRIBUTE_KEY = "panelType";

  public static final String INFO_PANEL_TYPE = "info";

  public static final String WARNING_PANEL_TYPE = "warning";

  public PanelMessageFormField(
      final String id,
      final String label,
      final String helpText,
      final String panelType)
  {
    super(id, label, helpText, false);
    getAttributes().put(PANEL_TYPE_ATTRIBUTE_KEY, panelType);
  }

  public PanelMessageFormField(
      final String id,
      final String helpText,
      final String panelType)
  {
    super(id, null, helpText, false);
    getAttributes().put(PANEL_TYPE_ATTRIBUTE_KEY, panelType);
  }

  @Override
  public String getType() {
    return "panelMessage";
  }
}
