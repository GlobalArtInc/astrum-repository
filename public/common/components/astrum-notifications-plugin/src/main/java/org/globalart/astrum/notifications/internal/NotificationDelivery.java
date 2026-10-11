package org.globalart.astrum.notifications.internal;

import java.util.Optional;
import java.util.concurrent.RejectedExecutionException;
import java.util.concurrent.ScheduledExecutorService;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.concurrent.TimeUnit.MILLISECONDS;

/**
 * Sends messages in the background. A message the chat could not take for the moment (rate limit, server error,
 * network failure) is sent again after the delay the chat asked for, or after an exponential backoff, until it has
 * been tried {@code maxAttempts} times.
 */
public class NotificationDelivery
{
  private static final Logger log = LoggerFactory.getLogger(NotificationDelivery.class);

  private final NotificationChannelStore store;

  private final NotificationTransport transport;

  private final ScheduledExecutorService executor;

  private final long backoffMillis;

  private final int maxAttempts;

  public NotificationDelivery(
      final NotificationChannelStore store,
      final NotificationTransport transport,
      final ScheduledExecutorService executor,
      final long backoffMillis,
      final int maxAttempts)
  {
    checkArgument(backoffMillis >= 0, "backoffMillis must not be negative");
    checkArgument(maxAttempts > 0, "maxAttempts must be positive");
    this.store = checkNotNull(store);
    this.transport = checkNotNull(transport);
    this.executor = checkNotNull(executor);
    this.backoffMillis = backoffMillis;
    this.maxAttempts = maxAttempts;
  }

  public void submit(final String channelId, final String text) {
    try {
      executor.execute(() -> attempt(channelId, text, 1));
    }
    catch (RejectedExecutionException e) {
      log.warn("Dropped a notification for channel {}: delivery is stopped", channelId);
    }
  }

  /**
   * @return how long to wait before attempt {@code attempt + 1}, or a negative number when there is none
   */
  long retryDelay(final int attempt, final NotificationDeliveryException failure) {
    if (!failure.isRetryable() || attempt >= maxAttempts) {
      return -1;
    }
    if (failure.getRetryAfterMillis() > 0) {
      return failure.getRetryAfterMillis();
    }
    return backoffMillis << Math.min(attempt - 1, 20);
  }

  private void attempt(final String channelId, final String text, final int attempt) {
    Optional<NotificationChannel> found = store.find(channelId).filter(NotificationChannel::isEnabled);
    if (found.isEmpty()) {
      return;
    }
    NotificationChannel channel = found.get();
    try {
      NotificationSecrets secrets = store.secrets(channel);
      if (secrets == null) {
        log.warn("Notification channel {} has no bot token or webhook URL", channel.getName());
        return;
      }
      transport.send(channel, secrets, text);
      log.debug("Sent a notification to channel {}", channel.getName());
    }
    catch (NotificationDeliveryException e) {
      long delay = retryDelay(attempt, e);
      if (delay < 0) {
        log.warn("Could not send a notification to channel {} (attempt {}): {}", channel.getName(), attempt,
            e.getMessage());
        return;
      }
      log.info("Could not send a notification to channel {} (attempt {}), retrying in {} ms: {}", channel.getName(),
          attempt, delay, e.getMessage());
      try {
        executor.schedule(() -> attempt(channelId, text, attempt + 1), delay, MILLISECONDS);
      }
      catch (RejectedExecutionException rejected) {
        log.warn("Dropped a notification for channel {}: delivery is stopped", channel.getName());
      }
    }
    catch (RuntimeException e) {
      log.warn("Could not send a notification to channel {}", channel.getName(), e);
    }
  }
}
