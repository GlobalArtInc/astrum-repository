
package com.globalart.astrum.ssl.plugin.internal;

import org.springframework.beans.factory.annotation.Autowired;
import javax.net.ssl.SSLContext;

import org.globalart.astrum.httpclient.SSLContextSelector;
import org.globalart.astrum.ssl.TrustStore;

import org.apache.http.protocol.HttpContext;
import org.springframework.context.annotation.Lazy;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * An {@link SSLContextSelector} that will make use of Astrum SSL TrustStore depending on value of
 * {@link SSLContextSelector#USE_TRUST_STORE} attribute.
 *
 * @since 3.0
 */
@Component
public class HttpContextAttributeSSLContextSelector
    implements SSLContextSelector
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final TrustStore trustStore;

  @Autowired
  public HttpContextAttributeSSLContextSelector(@Lazy final TrustStore trustStore) {
    this.trustStore = checkNotNull(trustStore);
  }

  @Override
  public SSLContext select(final HttpContext context) {
    Object useTrustStore = context.getAttribute(SSLContextSelector.USE_TRUST_STORE);
    if (Boolean.TRUE.equals(useTrustStore)) {
      return trustStore.getSSLContext();
    }
    return null;
  }
}
