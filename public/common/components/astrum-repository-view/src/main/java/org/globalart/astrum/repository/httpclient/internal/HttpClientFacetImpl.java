
package org.globalart.astrum.repository.httpclient.internal;

import java.io.IOException;
import java.util.Base64;
import java.util.List;
import java.util.Map;
import javax.annotation.Nullable;

import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.crypto.internal.error.CipherException;
import org.globalart.astrum.common.event.EventHelper;
import org.globalart.astrum.common.stateguard.Guarded;
import org.globalart.astrum.distributed.event.service.api.common.RepositoryRemoteConnectionStatusEvent;
import org.globalart.astrum.httpclient.HttpClientManager;
import org.globalart.astrum.httpclient.config.BearerTokenAuthenticationConfiguration;
import org.globalart.astrum.httpclient.config.ConfigurationCustomizer;
import org.globalart.astrum.httpclient.config.HttpClientConfiguration;
import org.globalart.astrum.httpclient.config.HttpClientConfigurationChangedEvent;
import org.globalart.astrum.httpclient.config.UsernameAuthenticationConfiguration;
import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.config.ConfigurationFacet;
import org.globalart.astrum.repository.httpclient.AutoBlockConfiguration;
import org.globalart.astrum.repository.httpclient.ContentCompressionStrategy;
import org.globalart.astrum.repository.httpclient.HttpClientConfig;
import org.globalart.astrum.repository.httpclient.HttpClientFacet;
import org.globalart.astrum.repository.httpclient.NormalizationStrategy;
import org.globalart.astrum.repository.httpclient.OutboundRequestMetricRecorder;
import org.globalart.astrum.repository.httpclient.RemoteConnectionStatus;
import org.globalart.astrum.repository.httpclient.RemoteConnectionStatusEvent;
import org.globalart.astrum.repository.httpclient.RemoteConnectionStatusObserver;

import com.google.common.annotations.VisibleForTesting;
import com.google.common.eventbus.Subscribe;
import org.springframework.beans.factory.annotation.Autowired;
import org.apache.http.Header;
import org.apache.http.HttpHost;
import org.apache.http.client.HttpClient;
import org.apache.http.client.RedirectStrategy;
import org.apache.http.impl.client.CloseableHttpClient;
import org.apache.http.impl.client.TargetAuthenticationStrategy;
import org.apache.http.message.BasicHeader;
import org.joda.time.DateTime;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.lang.String.format;
import static java.nio.charset.StandardCharsets.ISO_8859_1;
import static java.util.Optional.ofNullable;
import static org.apache.http.HttpHeaders.AUTHORIZATION;
import static org.globalart.astrum.repository.FacetSupport.State.STARTED;
import static org.globalart.astrum.repository.httpclient.RemoteConnectionStatusType.AUTO_BLOCKED_UNAVAILABLE;
import static org.globalart.astrum.repository.httpclient.RemoteConnectionStatusType.UNINITIALISED;

/**
 * Default {@link HttpClientFacet} implementation.
 *
 * @since 3.0
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class HttpClientFacetImpl
    extends FacetSupport
    implements HttpClientFacet, RemoteConnectionStatusObserver
{
  private static final String DEFAULT = "default";

  private final HttpClientManager httpClientManager;

  @VisibleForTesting
  static final String CONFIG_KEY = "httpclient";

  private final Map<String, AutoBlockConfiguration> autoBlockConfiguration;

  private final Map<String, RedirectStrategy> redirectStrategy;

  private final Map<String, NormalizationStrategy> normalizationStrategies;

  private final Map<String, ContentCompressionStrategy> contentCompressionStrategies;

  private final Map<String, TargetAuthenticationStrategy> authenticationStrategies;

  private final OutboundRequestMetricRecorder outboundRequestRecorder;

  private String unencryptedPassword;

  private HttpClientConfig config;

  @VisibleForTesting
  BlockingHttpClient httpClient;

  @Autowired
  public HttpClientFacetImpl(
      final HttpClientManager httpClientManager,
      final List<AutoBlockConfiguration> autoBlockConfigurationList,
      final List<RedirectStrategy> redirectStrategyList,
      final List<NormalizationStrategy> normalizationStrategiesList,
      final List<ContentCompressionStrategy> contentCompressionStrategiesList,
      final List<TargetAuthenticationStrategy> authenticationStrategiesList,
      @Nullable final OutboundRequestMetricRecorder outboundRequestRecorder)
  {
    this.httpClientManager = checkNotNull(httpClientManager);
    this.autoBlockConfiguration = QualifierUtil.buildQualifierBeanMap(checkNotNull(autoBlockConfigurationList));
    this.redirectStrategy = QualifierUtil.buildQualifierBeanMap(checkNotNull(redirectStrategyList));
    this.normalizationStrategies = QualifierUtil.buildQualifierBeanMap(checkNotNull(normalizationStrategiesList));
    this.contentCompressionStrategies =
        QualifierUtil.buildQualifierBeanMap(checkNotNull(contentCompressionStrategiesList));
    this.authenticationStrategies = QualifierUtil.buildQualifierBeanMap(checkNotNull(authenticationStrategiesList));
    this.outboundRequestRecorder = outboundRequestRecorder;
  }

  @VisibleForTesting
  HttpClientFacetImpl(
      final HttpClientManager httpClientManager,
      final List<AutoBlockConfiguration> autoBlockConfiguration,
      final List<RedirectStrategy> redirectStrategy,
      final List<NormalizationStrategy> normalizationStrategy,
      final List<ContentCompressionStrategy> contentCompressionStrategies,
      final List<TargetAuthenticationStrategy> authenticationStrategies,
      final HttpClientConfig config)
  {
    this(httpClientManager, autoBlockConfiguration, redirectStrategy, normalizationStrategy,
        contentCompressionStrategies, authenticationStrategies, (OutboundRequestMetricRecorder) null);
    this.config = checkNotNull(config);
    checkNotNull(this.autoBlockConfiguration.get(DEFAULT));
  }

  /**
   * Constructor without outbound request metric recording.
   */
  @VisibleForTesting
  HttpClientFacetImpl(
      final HttpClientManager httpClientManager,
      final List<AutoBlockConfiguration> autoBlockConfigurationList,
      final List<RedirectStrategy> redirectStrategyList,
      final List<NormalizationStrategy> normalizationStrategiesList,
      final List<ContentCompressionStrategy> contentCompressionStrategiesList,
      final List<TargetAuthenticationStrategy> authenticationStrategiesList)
  {
    this(httpClientManager, autoBlockConfigurationList, redirectStrategyList, normalizationStrategiesList,
        contentCompressionStrategiesList, authenticationStrategiesList, (OutboundRequestMetricRecorder) null);
  }

  @Override
  protected void doValidate(final Configuration configuration) throws Exception {
    facet(ConfigurationFacet.class).validateSection(configuration, CONFIG_KEY, HttpClientConfig.class);
  }

  @Override
  protected void doConfigure(final Configuration configuration) throws Exception {
    config = facet(ConfigurationFacet.class).readSection(configuration, CONFIG_KEY, HttpClientConfig.class);
    resetPasswordCache();
    log.debug("Config: {}", config);

    createHttpClient();
  }

  @Override
  protected void doDestroy() throws Exception {
    config = null;
  }

  @Override
  protected void doStop() throws Exception {
    closeHttpClient();
  }

  @Override
  @Guarded(by = STARTED)
  public HttpClient getHttpClient() {
    return checkNotNull(httpClient);
  }

  @Nullable
  @Override
  @Guarded(by = STARTED)
  public Header createBasicAuthHeader() {
    if (config.authentication instanceof UsernameAuthenticationConfiguration) {
      UsernameAuthenticationConfiguration userAuth = (UsernameAuthenticationConfiguration) config.authentication;

      if (unencryptedPassword == null) {
        try {
          unencryptedPassword = new String(userAuth.getPassword().decrypt());
        }
        catch (CipherException e) {
          log.error("Cannot create Basic Auth header for repository '{}': password decryption failed. " +
              "Update the repository configuration with valid credentials.",
              getRepository().getName(), e);
          return null;
        }
      }

      String auth = format("%1$s:%2$s", userAuth.getUsername(), unencryptedPassword);

      byte[] encodedAuth = Base64.getEncoder().encode(auth.getBytes(ISO_8859_1));

      String authHeader = "Basic " + new String(encodedAuth, ISO_8859_1);

      return new BasicHeader(AUTHORIZATION, authHeader);
    }
    else {
      log.debug("Basic auth header cannot be created for auth config of {}", config.authentication);
      return null;
    }
  }

  @Override
  public String getBearerToken() {
    if (config.authentication != null &&
        BearerTokenAuthenticationConfiguration.TYPE.equals(config.authentication.getType())) {
      BearerTokenAuthenticationConfiguration bearerTokenAuthenticationConfiguration =
          ((BearerTokenAuthenticationConfiguration) config.authentication);
      return new String(bearerTokenAuthenticationConfiguration.getBearerToken().decrypt());
    }
    return null;
  }

  @Override
  @Guarded(by = STARTED)
  public RemoteConnectionStatus getStatus() {
    return httpClient.getStatus();
  }

  @Override
  @Guarded(by = STARTED)
  public void setStatus(final RemoteConnectionStatus status) {
    httpClient.setRemoteConnectionStatus(status);
  }

  @Override
  @Guarded(by = STARTED)
  public void recordConnectionFailure(final IOException failure, final HttpHost target) {
    httpClient.recordConnectionFailure(failure, target);
  }

  @Subscribe
  public void on(final HttpClientConfigurationChangedEvent event) throws IOException {
    closeHttpClient();
    createHttpClient();
  }

  @Override
  public void onStatusChanged(final RemoteConnectionStatus oldStatus, final RemoteConnectionStatus newStatus) {
    logStatusChange(oldStatus, newStatus);
    getEventManager().post(new RemoteConnectionStatusEvent(newStatus, getRepository()));

    distributeRepositoryConnectionStatusChangedEvent(getRepository().getName(), newStatus);
  }

  private void logStatusChange(final RemoteConnectionStatus oldStatus, final RemoteConnectionStatus newStatus) {
    if (log.isInfoEnabled()) {
      if (oldStatus.getType() == AUTO_BLOCKED_UNAVAILABLE && newStatus.getType() == AUTO_BLOCKED_UNAVAILABLE) {
        logAutoBlockTimeIncreased(oldStatus, newStatus);
      }
      else if (oldStatus.getType() == UNINITIALISED) {
        log.info("Remote connection status of repository {} set to {}.", getRepository().getName(),
            newStatus.getDescription());
      }
      else {
        logStatusUpdated(oldStatus, newStatus);
      }
    }
  }

  private void logAutoBlockTimeIncreased(
      final RemoteConnectionStatus oldStatus,
      final RemoteConnectionStatus newStatus)
  {
    String message = "Repository status for {} continued as {} until {} - reason {} (previous reason was {})";
    log.info(message,
        getRepository().getName(),
        newStatus.getType(),
        newStatus.getBlockedUntil(),
        statusReason(newStatus),
        statusReason(oldStatus));
  }

  private void logStatusUpdated(final RemoteConnectionStatus oldStatus, final RemoteConnectionStatus newStatus) {
    String message = "Repository status for {} changed from {} to {}{} - reason {}";
    log.info(message,
        getRepository().getName(),
        oldStatus.getType(),
        newStatus.getType(),
        statusBlockedUntil(newStatus),
        statusReason(newStatus));
  }

  private static String statusBlockedUntil(final RemoteConnectionStatus status) {
    if (status.getBlockedUntil() != null) {
      return format(" until %s", status.getBlockedUntil());
    }
    return "";
  }

  private static String statusReason(final RemoteConnectionStatus status) {
    return format("%s for %s",
        status.getReason() != null ? status.getReason() : "n/a",
        status.getRequestUrl() != null ? status.getRequestUrl() : "n/a");
  }

  private void createHttpClient() {
    HttpClientConfiguration delegateConfig = getHttpClientConfiguration(httpClientManager, config);

    CloseableHttpClient delegate;
    try {
      delegate = httpClientManager.create(new ConfigurationCustomizer(delegateConfig));
    }
    catch (CipherException e) {
      log.debug("Failed to create HTTP client for repository '{}' while applying repository HTTP client authentication",
          getRepository().getName(), e);
      throw e;
    }

    boolean online = getRepository().getConfiguration().isOnline();
    // wrap delegate with auto-block aware client
    httpClient = new BlockingHttpClient(delegate, config, this, online, getAutoBlockConfiguration(),
        outboundRequestRecorder);
    log.debug("Created HTTP client: {}", httpClient);
  }

  protected HttpClientConfiguration getHttpClientConfiguration(
      final HttpClientManager httpClientManager,
      final HttpClientConfig config)
  {
    // construct http client delegate
    HttpClientConfiguration delegateConfig = httpClientManager.newConfiguration();
    delegateConfig.setConnection(config.connection);
    delegateConfig.setAuthentication(config.authentication);
    delegateConfig.setRedirectStrategy(getRedirectStrategy());
    setNormalizationStrategy(delegateConfig);
    setContentCompressionStrategy(delegateConfig);
    setAuthenticationStrategy(delegateConfig);
    return delegateConfig;
  }

  private AutoBlockConfiguration getAutoBlockConfiguration() {
    AutoBlockConfiguration config = this.autoBlockConfiguration.get(getRepository().getFormat().getValue());

    if (config == null) {
      config = autoBlockConfiguration.get(DEFAULT);
    }

    return config;
  }

  protected RedirectStrategy getRedirectStrategy() {
    RedirectStrategy strategy = this.redirectStrategy.get(getRepository().getFormat().getValue());

    if (strategy == null) {
      strategy = this.redirectStrategy.get(DEFAULT);
    }

    return strategy;
  }

  protected void setNormalizationStrategy(final HttpClientConfiguration delegateConfig) {
    ofNullable(normalizationStrategies.get(getRepository().getFormat().getValue()))
        .ifPresent(strategy -> delegateConfig.setNormalizeUri(strategy.shouldNormalizeUri()));
  }

  protected void setContentCompressionStrategy(final HttpClientConfiguration delegateConfig) {
    ofNullable(contentCompressionStrategies.get(getRepository().getFormat().getValue()))
        .ifPresent(strategy -> delegateConfig
            .setDisableContentCompression(strategy.shouldDisableContentCompression(getRepository().getName())));
  }

  private void setAuthenticationStrategy(final HttpClientConfiguration delegateConfig) {
    ofNullable(authenticationStrategies.get(getRepository().getFormat().getValue()))
        .ifPresent(delegateConfig::setAuthenticationStrategy);
  }

  private void closeHttpClient() throws IOException {
    log.debug("Closing HTTP client: {}", httpClient);
    httpClient.close();
    httpClient = null;
  }

  private void distributeRepositoryConnectionStatusChangedEvent(
      final String repositoryName,
      final RemoteConnectionStatus status)
  {
    if (EventHelper.isReplicating()) {
      log.debug("Originally triggered by remote event");
      return;
    }

    log.debug("Distribute repository block changed event: repository={}:{}", repositoryName, status.getType());

    long blockUntilMillis = status.getBlockedUntil() == null
        ? DateTime.now().getMillis()
        : status.getBlockedUntil().getMillis();

    RepositoryRemoteConnectionStatusEvent event = new RepositoryRemoteConnectionStatusEvent(
        repositoryName,
        status.getType().ordinal(),
        status.getReason(),
        blockUntilMillis,
        status.getRequestUrl());

    getEventManager().post(event);
  }

  private void resetPasswordCache() {
    if (config.authentication instanceof UsernameAuthenticationConfiguration) {
      UsernameAuthenticationConfiguration userAuth = (UsernameAuthenticationConfiguration) config.authentication;
      try {
        unencryptedPassword = new String(userAuth.getPassword().decrypt());
      }
      catch (CipherException e) {
        log.warn("Failed to decrypt password for repository '{}': {}. " +
            "Repository will start but authentication will not work until password is updated.",
            getRepository().getName(), e.getMessage());
        unencryptedPassword = null;
      }
    }
    else {
      unencryptedPassword = null;
    }
  }
}
