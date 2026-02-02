
package org.globalart.astrum.repository.view.handlers;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.collect.AttributeKey;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;

import com.google.common.base.Stopwatch;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Simple timing handler.
 *
 * @since 3.0
 */
@Component
public class TimingHandler
    implements Handler
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final String ELAPSED_KEY = AttributeKey.get(TimingHandler.class, "elapsed");

  private final Handler meteringHandler;

  @Autowired
  public TimingHandler(@Qualifier("astrum.analytics.meteringHandler") @Nullable final Handler meteringHandler) {
    this.meteringHandler = meteringHandler;
  }

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    Stopwatch watch = Stopwatch.createStarted();

    try {
      if (meteringHandler != null) {
        context.insertHandler(meteringHandler);
      }
      return context.proceed();
    }
    finally {
      String elapsed = watch.toString();
      context.getAttributes().set(ELAPSED_KEY, elapsed);
      log.trace("Timing: {}", elapsed);
    }
  }
}
