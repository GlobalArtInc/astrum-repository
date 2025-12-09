
package org.globalart.astrum.internal.httpclient;

import org.apache.http.HttpResponse;
import org.apache.http.impl.client.DefaultConnectionKeepAliveStrategy;
import org.apache.http.protocol.HttpContext;

import static com.google.common.base.Preconditions.checkArgument;

/**
 * Astrum connection keep alive strategy, that differs from the HC4 default one only in one thing: when server does
 * not state timeout, it never says "indefinite" (meaning pool it forever), but instead a finite amount of time.
 *
 * @since 2.3
 */
public class AstrumConnectionKeepAliveStrategy
    extends DefaultConnectionKeepAliveStrategy
{
  // FIXME: Clean up names, used as max here, but config passed in isn't a "max"

  /**
   * The max duration for how long to pool a connection in milliseconds. Used as default too, instead of
   * "indefinite" case.
   */
  private final long maxKeepAliveDuration;

  /**
   * @param maxKeepAliveDuration the max duration in millis for how long to pool the connection.
   */
  AstrumConnectionKeepAliveStrategy(final long maxKeepAliveDuration) {
    checkArgument(maxKeepAliveDuration > -1, "maxKeepAliveDuration must be 0 or higher, but is set to %s",
        maxKeepAliveDuration);
    this.maxKeepAliveDuration = maxKeepAliveDuration;
  }

  /**
   * Returns the duration of time which this connection can be safely kept
   * idle. Astrum by default does not "believe" much to remote servers, and will never
   * keep connection pooled "forever", nor will keep it pooled for unreasonable long time.
   *
   * @return the duration of time which this connection can be safely kept idle in pool.
   */
  public long getKeepAliveDuration(final HttpResponse response, final HttpContext context) {
    // ask super class
    final long result = super.getKeepAliveDuration(response, context);
    if (result < 0) {
      // if "indefinite", use default
      return maxKeepAliveDuration;
    }
    else {
      // else "cap" it with max
      return Math.min(result, maxKeepAliveDuration);
    }
  }
}
