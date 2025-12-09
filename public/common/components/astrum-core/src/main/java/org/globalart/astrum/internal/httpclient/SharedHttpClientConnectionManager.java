
package org.globalart.astrum.internal.httpclient;

import java.util.List;
import java.util.concurrent.TimeUnit;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.time.Time;
import org.globalart.astrum.common.lifecycle.Lifecycle;
import org.globalart.astrum.httpclient.SSLContextSelector;

import org.apache.http.config.Registry;
import org.apache.http.config.RegistryBuilder;
import org.apache.http.config.SocketConfig;
import org.apache.http.conn.socket.ConnectionSocketFactory;
import org.apache.http.conn.socket.PlainConnectionSocketFactory;
import org.apache.http.impl.conn.DefaultHttpClientConnectionOperator;
import org.apache.http.impl.conn.PoolingHttpClientConnectionManager;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Value;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.ManagedLifecycleManager.isShuttingDown;
import static org.globalart.astrum.httpclient.HttpSchemes.HTTP;
import static org.globalart.astrum.httpclient.HttpSchemes.HTTPS;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

// TODO: Restore JMX support for httpclient bits

/**
 * Shared {@link PoolingHttpClientConnectionManager}.
 *
 * @since 3.0
 */
@Component
@Qualifier("shared")
public class SharedHttpClientConnectionManager
    extends PoolingHttpClientConnectionManager
    implements Lifecycle
{
  private static final Logger log = LoggerFactory.getLogger(SharedHttpClientConnectionManager.class);

  private final Time connectionPoolIdleTime;

  private final Time connectionPoolEvictingDelayTime;

  private ConnectionEvictionThread evictionThread;

  @Autowired
  public SharedHttpClientConnectionManager(
      final List<SSLContextSelector> sslContextSelectors,
      @Value("${astrum.httpclient.connectionpool.size:20}") final int connectionPoolSize,
      @Value("${astrum.httpclient.connectionpool.maxSize:200}") final int connectionPoolMaxSize,
      @Value("${astrum.httpclient.connectionpool.idleTime:30s}") final Time connectionPoolIdleTime,
      @Value("${astrum.httpclient.connectionpool.evictingDelayTime:5s}") final Time connectionPoolEvictingDelayTime,
      @Value("${astrum.httpclient.connectionpool.validateAfterInactivityTime:2s}") final Time connectionPoolValidateAfterInactivityTime,
      @Value("${astrum.httpclient.connectionpool.default.requestTimeout:20s}") final Time defaultSocketTimeout)
  {
    super(
        new DefaultHttpClientConnectionOperator(createRegistry(sslContextSelectors), null, null),
        null,
        connectionPoolIdleTime.toMillis(),
        TimeUnit.MILLISECONDS);

    setMaxTotal(connectionPoolMaxSize);
    log.debug("Connection pool max-size: {}", connectionPoolMaxSize);

    setDefaultMaxPerRoute(Math.min(connectionPoolSize, connectionPoolMaxSize));
    log.debug("Connection pool size: {}", connectionPoolSize);

    this.connectionPoolIdleTime = checkNotNull(connectionPoolIdleTime);
    this.connectionPoolEvictingDelayTime = checkNotNull(connectionPoolEvictingDelayTime);
    setValidateAfterInactivity(connectionPoolValidateAfterInactivityTime.toMillisI());
    log.debug("Connection pool idle-time: {}, evicting delay: {}, validate after inactivity: {}",
        connectionPoolIdleTime, connectionPoolEvictingDelayTime, connectionPoolValidateAfterInactivityTime);

    setDefaultSocketConfig(SocketConfig.custom().setSoTimeout(defaultSocketTimeout.toMillisI()).build());
    log.debug("Default socket timeout {}", defaultSocketTimeout);
  }

  private static Registry<ConnectionSocketFactory> createRegistry(final List<SSLContextSelector> sslContextSelectors) {
    RegistryBuilder<ConnectionSocketFactory> builder = RegistryBuilder.create();
    builder.register(HTTP, PlainConnectionSocketFactory.getSocketFactory());
    builder.register(HTTPS, new AstrumSSLConnectionSocketFactory(sslContextSelectors));
    return builder.build();
  }

  /**
   * Do nothing in order to avoid unwanted shutdown of shared connection manager.
   *
   * @see #stop()
   */
  @Override
  public void shutdown() {
    // empty
  }

  //
  // Lifecycle
  //

  // TODO: Maybe better to delegate to use lifecycle framework as LifecycleAware

  @Override
  public void start() throws Exception {
    evictionThread = new ConnectionEvictionThread(this, connectionPoolIdleTime, connectionPoolEvictingDelayTime);
    evictionThread.start();
  }

  @Override
  public void stop() throws Exception {
    evictionThread.interrupt();
    evictionThread = null;

    // underlying pool cannot be restarted, so avoid shutting it down when bouncing the service
    if (isShuttingDown()) {
      super.shutdown();
    }
  }
}
