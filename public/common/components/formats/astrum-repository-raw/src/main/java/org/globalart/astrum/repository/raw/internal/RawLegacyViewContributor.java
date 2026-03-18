
package org.globalart.astrum.repository.raw.internal;

import java.util.regex.Pattern;

import org.globalart.astrum.repository.httpbridge.LegacyViewConfiguration;
import org.globalart.astrum.repository.httpbridge.LegacyViewContributor;
import org.springframework.stereotype.Component;

/**
 * Legacy view contributor for Raw.
 *
 * @since 3.7
 */
@Component
public class RawLegacyViewContributor
    implements LegacyViewContributor
{
  @Override
  public LegacyViewConfiguration contribute() {
    return new LegacyViewConfiguration()
    {
      @Override
      public String getFormat() {
        return RawFormat.NAME;
      }

      @Override
      public Pattern getRequestPattern() {
        return Pattern.compile("/content/sites/.*");
      }
    };
  }
}
