package org.globalart.astrum.notifications.internal;

import java.util.List;
import java.util.Optional;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.TimeUnit;

import org.junit.After;
import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.empty;
import static org.hamcrest.Matchers.is;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

public class NotificationDeliveryTest
{
  private final ScheduledExecutorService executor = Executors.newScheduledThreadPool(1);

  private NotificationChannelStore store;

  private final List<String> attempts = new CopyOnWriteArrayList<>();

  private final List<Long> attemptTimes = new CopyOnWriteArrayList<>();

  private final List<NotificationDeliveryException> failures = new CopyOnWriteArrayList<>();

  private final NotificationChannel channel = new NotificationChannel();

  private NotificationDelivery delivery;

  @Before
  public void setUp() {
    channel.setId("c1");
    channel.setName("releases");
    channel.setType(NotificationChannel.TELEGRAM);
    store = mock(NotificationChannelStore.class);
    when(store.find("c1")).thenAnswer(invocation -> Optional.of(channel.copy()));
    when(store.secrets(any())).thenReturn(NotificationSecrets.of("1:token"));
    NotificationTransport transport = (target, secret, text) -> {
      attempts.add(text);
      attemptTimes.add(System.nanoTime());
      if (!failures.isEmpty()) {
        throw failures.remove(0);
      }
    };
    delivery = new NotificationDelivery(store, transport, executor, 10, NotificationDispatcher.MAX_ATTEMPTS);
  }

  @After
  public void tearDown() {
    executor.shutdownNow();
  }

  private void awaitAttempts(final int count) throws InterruptedException {
    long deadline = System.currentTimeMillis() + 10_000;
    while (attempts.size() < count && System.currentTimeMillis() < deadline) {
      Thread.sleep(10);
    }
    Thread.sleep(200);
  }

  @Test
  public void sendsOnce() throws Exception {
    delivery.submit("c1", "hello");

    awaitAttempts(1);
    assertThat(attempts, contains("hello"));
  }

  @Test
  public void retriesRateLimitedMessagesAfterTheRequestedDelay() throws Exception {
    failures.add(new NotificationDeliveryException("Telegram answered 429", 429, true, 300));

    delivery.submit("c1", "hello");

    awaitAttempts(2);
    assertThat(attempts, contains("hello", "hello"));
    long waited = TimeUnit.NANOSECONDS.toMillis(attemptTimes.get(1) - attemptTimes.get(0));
    assertThat(waited >= 290, is(true));
  }

  @Test
  public void retriesServerErrorsThreeTimesAtMost() throws Exception {
    for (int i = 0; i < 10; i++) {
      failures.add(new NotificationDeliveryException("Telegram answered 502", 502, true, 0));
    }

    delivery.submit("c1", "hello");

    awaitAttempts(NotificationDispatcher.MAX_ATTEMPTS);
    Thread.sleep(300);
    assertThat(attempts.size(), is(4));
  }

  @Test
  public void doesNotRetryRejectedMessages() throws Exception {
    failures.add(new NotificationDeliveryException("Telegram answered 400", 400, false, 0));

    delivery.submit("c1", "hello");

    awaitAttempts(1);
    Thread.sleep(200);
    assertThat(attempts.size(), is(1));
  }

  @Test
  public void backoffDoublesUnlessTheChatAsksForADelay() {
    NotificationDeliveryException serverError = new NotificationDeliveryException("502", 502, true, 0);
    NotificationDeliveryException rateLimit = new NotificationDeliveryException("429", 429, true, 5000);
    NotificationDeliveryException rejected = new NotificationDeliveryException("400", 400, false, 0);

    assertThat(delivery.retryDelay(1, serverError), is(10L));
    assertThat(delivery.retryDelay(2, serverError), is(20L));
    assertThat(delivery.retryDelay(3, serverError), is(40L));
    assertThat(delivery.retryDelay(4, serverError), is(-1L));
    assertThat(delivery.retryDelay(1, rateLimit), is(5000L));
    assertThat(delivery.retryDelay(1, rejected), is(-1L));
  }

  @Test
  public void skipsChannelsThatWereDisabledOrDeleted() throws Exception {
    channel.setEnabled(false);
    delivery.submit("c1", "hello");
    when(store.find(anyString())).thenReturn(Optional.empty());
    delivery.submit("c2", "hello");

    Thread.sleep(300);
    assertThat(attempts, is(empty()));
  }
}
