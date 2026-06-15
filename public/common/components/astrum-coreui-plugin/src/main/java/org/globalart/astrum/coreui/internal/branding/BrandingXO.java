
package org.globalart.astrum.coreui.internal.branding;

/**
 * Branding exchange object.
 *
 * @since 3.0
 */
public class BrandingXO
{
  private boolean headerEnabled;

  private String headerHtml;

  private boolean footerEnabled;

  private String footerHtml;

  public BrandingXO() {
  }

  public BrandingXO(
      final boolean headerEnabled,
      final String headerHtml,
      final boolean footerEnabled,
      final String footerHtml)
  {
    this.headerEnabled = headerEnabled;
    this.headerHtml = headerHtml;
    this.footerEnabled = footerEnabled;
    this.footerHtml = footerHtml;
  }

  public boolean isHeaderEnabled() {
    return headerEnabled;
  }

  public void setHeaderEnabled(boolean headerEnabled) {
    this.headerEnabled = headerEnabled;
  }

  public String getHeaderHtml() {
    return headerHtml;
  }

  public void setHeaderHtml(String headerHtml) {
    this.headerHtml = headerHtml;
  }

  public boolean isFooterEnabled() {
    return footerEnabled;
  }

  public void setFooterEnabled(Boolean footerEnabled) {
    this.footerEnabled = footerEnabled;
  }

  public String getFooterHtml() {
    return footerHtml;
  }

  public void setFooterHtml(String footerHtml) {
    this.footerHtml = footerHtml;
  }

  @Override
  public String toString() {
    return "BrandingXO{" +
        "headerEnabled=" + headerEnabled +
        ", headerHtml='" + headerHtml + '\'' +
        ", footerEnabled=" + footerEnabled +
        ", footerHtml='" + footerHtml + '\'' +
        '}';
  }
}
