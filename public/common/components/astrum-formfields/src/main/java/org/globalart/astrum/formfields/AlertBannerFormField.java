
package org.globalart.astrum.formfields;

public class AlertBannerFormField
    extends AbstractFormField<String>
{
  public AlertBannerFormField(Position position, final String title) {
    super("%sAlertBanner".formatted(position.toString().toLowerCase()), position.toString(), title, false);
  }

  @Override
  public String getType() {
    return "templateOnly";
  }

  public enum Position
  {
    TOP,
    BOTTOM
  }
}
