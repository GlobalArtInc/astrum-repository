
package org.globalart.astrum.common.event;

import com.google.common.eventbus.EventBus;
import com.google.common.eventbus.SubscriberExceptionContext;
import com.google.common.eventbus.SubscriberExceptionHandler;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Slf4j logging {@link SubscriberExceptionHandler}.
 *
 * @since 3.0
 */
final class Slf4jSubscriberExceptionHandler
    implements SubscriberExceptionHandler
{
  private final Logger logger;

  public Slf4jSubscriberExceptionHandler(final String identifier) {
    checkNotNull(identifier);
    this.logger = LoggerFactory.getLogger(EventBus.class.getName() + "." + identifier);
  }

  @Override
  public void handleException(final Throwable exception, final SubscriberExceptionContext context) {
    logger.error("Could not dispatch event {} to subscriber {} method [{}]",
        context.getEvent(), context.getSubscriber(), context.getSubscriberMethod(), exception);
  }
}
