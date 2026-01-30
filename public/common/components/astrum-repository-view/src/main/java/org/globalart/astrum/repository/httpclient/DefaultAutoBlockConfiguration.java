
package org.globalart.astrum.repository.httpclient;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static org.apache.http.HttpStatus.SC_UNAUTHORIZED;
import static org.globalart.astrum.repository.http.HttpStatus.PROXY_AUTHENTICATION_REQUIRED;

/**
 * Default configuration for auto-blocking.
 *
 * @since 3.14
 */
@Qualifier("default")
@Component
public class DefaultAutoBlockConfiguration
    implements AutoBlockConfiguration
{
  @Override
  public boolean shouldBlock(final int statusCode) {
    return statusCode == SC_UNAUTHORIZED || statusCode == PROXY_AUTHENTICATION_REQUIRED || statusCode >= 500;
  }
}
