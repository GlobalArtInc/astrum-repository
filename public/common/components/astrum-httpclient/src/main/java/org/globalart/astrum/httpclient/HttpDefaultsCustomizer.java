
package org.globalart.astrum.httpclient;

import org.globalart.astrum.common.time.Time;

public interface HttpDefaultsCustomizer
    extends HttpClientPlan.Customizer
{
  Time getRequestTimeout();

  int getRetryCount();
}
