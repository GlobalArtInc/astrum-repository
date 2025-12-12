
package org.globalart.astrum.utils.httpclient;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.capability.CapabilityReference;
import org.globalart.astrum.common.app.ApplicationVersion;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Generates the {@code User-Agent} header value.
 *
 * @since 3.0
 */
@Component
public class UserAgentGenerator
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String PAU = "; pau)";

  private static final String PAE = "; pae)";

  private static final String PAD = "; pad)";

  private final ApplicationVersion applicationVersion;

  private String value;

  private String edition;

  @Autowired
  public UserAgentGenerator(final ApplicationVersion applicationVersion) {
    this.applicationVersion = checkNotNull(applicationVersion);
  }

  public String generate() {
    // Cache platform details or re-cache if the edition has changed
    if (value == null || !applicationVersion.getEdition().equals(edition)) {
      // track edition for cache invalidation
      edition = applicationVersion.getEdition();

      value = String.format("Astrum/%s (%s; %s; %s; %s; %s)",
          applicationVersion.getVersion(),
          edition,
          System.getProperty("os.name"),
          System.getProperty("os.version"),
          System.getProperty("os.arch"),
          System.getProperty("java.version"));
    }

    return value;
  }

  public String buildUserAgentForAnalytics(CapabilityReference capabilityReference) {
    String ua = generate();
    if (capabilityReference == null) {
      return ua.replace(")", PAU);
    }
    else if (capabilityReference.context().isEnabled()) {
      return ua.replace(")", PAE);
    }
    else {
      return ua.replace(")", PAD);
    }
  }
}
