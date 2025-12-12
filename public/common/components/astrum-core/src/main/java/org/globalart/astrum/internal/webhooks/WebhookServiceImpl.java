
package org.globalart.astrum.internal.webhooks;

import java.io.IOException;
import java.net.SocketException;
import java.net.URI;
import java.security.InvalidKeyException;
import java.security.NoSuchAlgorithmException;
import java.util.List;
import java.util.concurrent.LinkedBlockingQueue;
import java.util.concurrent.ThreadPoolExecutor;
import java.util.concurrent.ThreadPoolExecutor.AbortPolicy;
import java.util.concurrent.TimeUnit;

import javax.annotation.Nullable;
import javax.crypto.Mac;
import javax.crypto.spec.SecretKeySpec;

import org.globalart.astrum.common.InternalAccessible;
import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.common.stateguard.Guarded;
import org.globalart.astrum.common.stateguard.InvalidStateException;
import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;
import org.globalart.astrum.thread.AstrumThreadFactory;
import org.globalart.astrum.validation.ssrf.AntiSsrfService;
import org.globalart.astrum.webhooks.Webhook;
import org.globalart.astrum.webhooks.WebhookRequest;
import org.globalart.astrum.webhooks.WebhookRequestSendEvent;
import org.globalart.astrum.webhooks.WebhookService;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import com.codahale.metrics.annotation.Gauge;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.SerializationFeature;
import com.google.common.annotations.VisibleForTesting;
import com.google.common.collect.ImmutableList;
import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import com.google.common.io.BaseEncoding;

import jakarta.inject.Provider;
import jakarta.validation.ValidationException;

import org.apache.http.HttpEntity;
import org.apache.http.HttpResponse;
import org.apache.http.StatusLine;
import org.apache.http.client.HttpResponseException;
import org.apache.http.client.methods.CloseableHttpResponse;
import org.apache.http.client.methods.HttpPost;
import org.apache.http.entity.ContentType;
import org.apache.http.entity.StringEntity;
import org.apache.http.impl.client.CloseableHttpClient;
import org.apache.http.util.EntityUtils;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;
import static java.lang.Thread.MIN_PRIORITY;
import static org.apache.commons.lang3.StringUtils.isEmpty;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.TASKS;
import static org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport.State.STARTED;

/**
 * Default {@link WebhookService} implementation.
 *
 * @since 3.1
 */
@Component
@ManagedLifecycle(phase = TASKS)
public class WebhookServiceImpl
    extends StateGuardLifecycleSupport
    implements WebhookService, EventAware, EventAware.Asynchronous
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String WEBHOOK_ID_HEADER = "X-Astrum-Webhook-ID";

  private static final String WEBHOOK_DELIVERY_HEADER = "X-Astrum-Webhook-Delivery";

  @VisibleForTesting
  static final String WEBHOOK_SIGNATURE_HEADER = "X-Astrum-Webhook-Signature";

  private static final String HMAC_SHA1 = "HmacSHA1";

  private static final BaseEncoding HEX = BaseEncoding.base16().lowerCase();

  private final ObjectMapper objectMapper = new ObjectMapper()
      .disable(SerializationFeature.WRITE_DATES_AS_TIMESTAMPS)
      .setSerializationInclusion(JsonInclude.Include.NON_NULL);

  private final Provider<CloseableHttpClient> httpClientProvider;

  private final AntiSsrfService antiSsrfService;

  private final List<Webhook> webhooks;

  private final ThreadPoolExecutor threadPoolExecutor;

  @Autowired
  public WebhookServiceImpl(
      final Provider<CloseableHttpClient> httpClientProvider,
      final AntiSsrfService antiSsrfService,
      final List<Webhook> webhooks,
      @Value("${astrum.webhook.pool.size:128}") final int poolSize)
  {
    this.httpClientProvider = checkNotNull(httpClientProvider);
    this.antiSsrfService = checkNotNull(antiSsrfService);
    this.webhooks = checkNotNull(webhooks);

    checkArgument(poolSize > 0, "Pool size must be greater than zero");
    this.threadPoolExecutor = new ThreadPoolExecutor(
        poolSize, // core-size
        poolSize, // max-size
        0L, // keep-alive
        TimeUnit.MILLISECONDS,
        new LinkedBlockingQueue<>(), // allow queueing up of requests
        new AstrumThreadFactory("webhookService", "requestRool", MIN_PRIORITY),
        new AbortPolicy());
  }

  /**
   * Attempt to extract response body as string.
   */
  @Nullable
  private static String extractResponseBody(final HttpResponse response) throws IOException {
    HttpEntity entity = response.getEntity();
    if (entity != null) {
      try {
        String body = EntityUtils.toString(entity);
        if (body != null && body.length() != 0 && !body.contains("<html")) {
          return body;
        }
      }
      finally {
        EntityUtils.consume(entity);
      }
    }
    return null;
  }

  /**
   * Generate HMAC signature (HEX encoded) of given body using secret as key.
   */
  private static String sign(
      final String body,
      final String secret) throws NoSuchAlgorithmException, InvalidKeyException
  {
    SecretKeySpec key = new SecretKeySpec(secret.getBytes(), HMAC_SHA1);
    Mac mac = Mac.getInstance(HMAC_SHA1);
    mac.init(key);
    byte[] bytes = mac.doFinal(body.getBytes());
    return HEX.encode(bytes);
  }

  @Override
  public List<Webhook> getWebhooks() {
    return ImmutableList.copyOf(webhooks);
  }

  //
  // Lifecycle
  //

  @Override
  protected void doStop() throws Exception {
    // Events received after stop will be rejected by @Guarded(by = STARTED) on queue(),
    // and the on() method catches InvalidStateException to log at DEBUG level.
    threadPoolExecutor.shutdown();
    try {
      if (!threadPoolExecutor.awaitTermination(30, TimeUnit.SECONDS)) {
        threadPoolExecutor.shutdownNow();
        log.warn("Webhook thread pool did not terminate gracefully within 30 seconds");
      }
    }
    catch (InterruptedException e) {
      threadPoolExecutor.shutdownNow();
      Thread.currentThread().interrupt();
    }
  }

  @Override
  @Guarded(by = STARTED)
  public void queue(final WebhookRequest request) {
    checkNotNull(request);
    threadPoolExecutor.execute(() -> {
      try {
        send(request);
      }
      catch (HttpResponseException e) {
        log.warn("Webhook endpoint returned error status for request: {} - {}",
            request, e.getMessage());
      }
      catch (SocketException e) {
        if (threadPoolExecutor.isShutdown()) {
          log.debug("Webhook request interrupted by shutdown: {}", request, e);
        }
        else {
          log.warn("Webhook request failed with socket error: {} - {}", request, e.getMessage());
        }
      }
      catch (Exception e) {
        log.error("Failed to send webhook request:{}", request, e);
      }
    });
  }

  /**
   * Asynchronous send handler.
   *
   * @see #queue(WebhookRequest)
   */
  @Subscribe
  @AllowConcurrentEvents
  @InternalAccessible
  void on(final WebhookRequestSendEvent event) {
    try {
      queue(event.getRequest());
    }
    catch (InvalidStateException e) {
      // Expected during shutdown - queue() is @Guarded(by = STARTED) and will reject
      // events after doStop() completes. Log at DEBUG to avoid noise during shutdown.
      log.debug("Webhook event received after shutdown, ignoring: {}", event.getRequest());
    }
  }

  @Override
  @Guarded(by = STARTED)
  public void send(final WebhookRequest request) throws Exception {
    checkNotNull(request);

    log.debug("Sending webhook request: {}", request);

    // Validate webhook URL doesn't point to private/local network (SSRF protection)
    URI url = request.getUrl();
    String host = url != null ? url.getHost() : null;
    if (host != null) {
      try {
        antiSsrfService.validateHost(host);
      }
      catch (ValidationException e) {
        log.warn("Webhook blocked by SSRF protection: {} - {}", url, e.getMessage());
        throw new IOException("Webhook URL blocked by SSRF protection: " + e.getMessage());
      }
    }

    Webhook webhook = request.getWebhook();
    String json = objectMapper.writeValueAsString(request.getPayload());

    HttpPost httpPost = new HttpPost(request.getUrl());
    httpPost.setHeader(WEBHOOK_ID_HEADER, webhook.getId());
    httpPost.setHeader(WEBHOOK_DELIVERY_HEADER, request.getId());
    // generate HMAC signature of body if secret is present
    if (!isEmpty(request.getSecret())) {
      httpPost.setHeader(WEBHOOK_SIGNATURE_HEADER, sign(json, request.getSecret()));
    }
    httpPost.setEntity(new StringEntity(json, ContentType.APPLICATION_JSON));

    log.debug("Sending POST request: {}", httpPost);
    try (CloseableHttpClient httpClient = httpClientProvider.get();
        CloseableHttpResponse putResponse = httpClient.execute(httpPost)) {

      StatusLine status = putResponse.getStatusLine();
      log.debug("Response status: {}", status);

      // on exceptional status throw exception
      int code = status.getStatusCode();
      if (code >= 300) {
        String message = extractResponseBody(putResponse);
        if (message == null) {
          message = status.getReasonPhrase();
        }
        throw new HttpResponseException(code, message);
      }
    }
  }

  @VisibleForTesting
  public boolean isCalmPeriod() {
    return threadPoolExecutor.getQueue().isEmpty() && threadPoolExecutor.getActiveCount() == 0;
  }

  @Gauge(name = "astrum.webhooks.service.executor.queueSize")
  public int webhookQueueSize() {
    return threadPoolExecutor.getQueue().size();
  }
}
