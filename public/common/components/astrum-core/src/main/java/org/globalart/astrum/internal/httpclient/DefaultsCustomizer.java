
package org.globalart.astrum.internal.httpclient;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.io.ByteSize;
import org.globalart.astrum.common.time.Time;
import org.globalart.astrum.httpclient.HttpClientPlan;
import org.globalart.astrum.httpclient.HttpDefaultsCustomizer;
import org.globalart.astrum.utils.httpclient.UserAgentGenerator;

import org.apache.http.client.config.CookieSpecs;
import org.apache.http.impl.client.StandardHttpRequestRetryHandler;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.context.annotation.Primary;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Applies defaults to {@link HttpClientPlan}.
 *
 * @since 3.0
 */
@Primary
@Component
public class DefaultsCustomizer
    implements HttpDefaultsCustomizer
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final UserAgentGenerator userAgentGenerator;

  private final Time requestTimeout;

  private final Time connectionRequestTimeout;

  private final Time keepAliveDuration;

  private final ByteSize bufferSize;

  private final int retryCount;

  @Autowired
  public DefaultsCustomizer(
      final UserAgentGenerator userAgentGenerator,
      @Value("${astrum.httpclient.requestTimeout:20s}") final Time requestTimeout,
      @Value("${astrum.httpclient.connectionRequestTimeout:30s}") final Time connectionRequestTimeout,
      @Value("${astrum.httpclient.keepAliveDuration:30s}") final Time keepAliveDuration,
      @Value("${astrum.httpclient.bufferSize:8k}") final ByteSize bufferSize,
      @Value("${astrum.httpclient.retryCount:2}") final int retryCount)
  {
    this.userAgentGenerator = checkNotNull(userAgentGenerator);

    this.requestTimeout = checkNotNull(requestTimeout);
    log.debug("Request timeout: {}", requestTimeout);

    this.connectionRequestTimeout = checkNotNull(connectionRequestTimeout);
    log.debug("Connection request timeout: {}", connectionRequestTimeout);

    this.keepAliveDuration = checkNotNull(keepAliveDuration);
    log.debug("Keep-alive duration: {}", keepAliveDuration);

    this.bufferSize = checkNotNull(bufferSize);
    log.debug("Buffer-size: {}", bufferSize);

    this.retryCount = checkNotNull(retryCount);
  }

  @Override
  public void customize(final HttpClientPlan plan) {
    checkNotNull(plan);

    plan.setUserAgentBase(userAgentGenerator.generate());

    plan.getClient().setKeepAliveStrategy(new AstrumConnectionKeepAliveStrategy(keepAliveDuration.toMillis()));
    plan.getClient().setRetryHandler(new StandardHttpRequestRetryHandler(retryCount, false));

    plan.getConnection().setBufferSize(bufferSize.toBytesI());

    plan.getRequest().setConnectionRequestTimeout(connectionRequestTimeout.toMillisI());
    plan.getRequest().setCookieSpec(CookieSpecs.IGNORE_COOKIES);
    plan.getRequest().setExpectContinueEnabled(false);

    int requestTimeoutMillis = requestTimeout.toMillisI();
    plan.getSocket().setSoTimeout(requestTimeoutMillis);
    plan.getRequest().setConnectTimeout(requestTimeoutMillis);
    plan.getRequest().setSocketTimeout(requestTimeoutMillis);
  }

  @Override
  public Time getRequestTimeout() {
    return requestTimeout;
  }

  @Override
  public int getRetryCount() {
    return retryCount;
  }
}
