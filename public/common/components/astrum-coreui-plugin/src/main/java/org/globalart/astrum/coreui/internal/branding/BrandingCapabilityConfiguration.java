
package org.globalart.astrum.coreui.internal.branding;

import java.util.Map;

import org.globalart.astrum.common.text.Strings2;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Configuration adapter for {@link BrandingCapability}.
 *
 * @since 3.0
 */
public class BrandingCapabilityConfiguration
    extends BrandingXO
{

  public static final String HEADER_ENABLED = "headerEnabled";

  public static final String HEADER_HTML = "headerHtml";

  public static final String FOOTER_ENABLED = "footerEnabled";

  public static final String FOOTER_HTML = "footerHtml";

  public BrandingCapabilityConfiguration(final Map<String, String> properties) {
    checkNotNull(properties);
    setHeaderEnabled(parseBoolean(properties.get(HEADER_ENABLED), false));
    setHeaderHtml(parseString(properties.get(HEADER_HTML)));
    setFooterEnabled(parseBoolean(properties.get(FOOTER_ENABLED), false));
    setFooterHtml(parseString(properties.get(FOOTER_HTML)));
  }

  private boolean parseBoolean(final String value, final boolean defaultValue) {
    if (!Strings2.isEmpty(value)) {
      return Boolean.parseBoolean(value);
    }
    return defaultValue;
  }

  private String parseString(final String value) {
    if (!Strings2.isEmpty(value)) {
      return value;
    }
    return null;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName()
        + "{"
        + "headerEnabled=" + isHeaderEnabled()
        + ", footerEnabled=" + isFooterEnabled()
        + "}";
  }
}
