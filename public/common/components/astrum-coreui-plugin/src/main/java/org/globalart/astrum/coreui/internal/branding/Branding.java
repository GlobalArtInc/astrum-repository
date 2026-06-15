
package org.globalart.astrum.coreui.internal.branding;

import java.util.Collections;
import java.util.Map;
import java.util.regex.Matcher;
import javax.annotation.Nullable;
import org.globalart.astrum.common.app.BaseUrlHolder;
import org.globalart.astrum.rapture.StateContributor;
import org.springframework.stereotype.Component;

/**
 * Branding state contributor.
 *
 * @since 3.0
 */
@Component
public class Branding
    implements StateContributor
{
  private BrandingCapabilityConfiguration config;

  @Override
  public Map<String, Object> getState() {
    if (config != null) {
      return Collections.singletonMap("branding",
          new BrandingXO(config.isHeaderEnabled(),
              interpolate(config.getHeaderHtml()),
              config.isFooterEnabled(),
              interpolate(config.getFooterHtml())));
    }
    return null;
  }

  @Nullable
  String interpolate(@Nullable final String html) {
    if (html != null) {
      return html.replaceAll(Matcher.quoteReplacement("$baseUrl"), BaseUrlHolder.getRelativePath());
    }
    return null;
  }

  public void set(final BrandingCapabilityConfiguration config) {
    this.config = config;
  }

  public void reset() {
    this.config = null;
  }
}
