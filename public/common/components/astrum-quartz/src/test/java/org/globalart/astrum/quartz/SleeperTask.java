
package org.globalart.astrum.quartz;

import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;

import org.globalart.astrum.scheduling.TaskSupport;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Primary;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * Simple sleeper task that is not cancelable.
 */
@Primary
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class SleeperTask
    extends TaskSupport
{
  public static final String RESULT_KEY = "result";

  static final String SLEEP_MILLIS_KEY = "sleepMillis";

  public static CountDownLatch meWait;

  public static CountDownLatch youWait;

  static Exception exception;

  public static void reset() {
    meWait = new CountDownLatch(1);
    youWait = new CountDownLatch(1);
    exception = null;
  }

  @Override
  protected String execute() throws Exception {
    youWait.countDown(); // task signals "started" to test

    while (!meWait.await(1L, TimeUnit.SECONDS)) { // test signals "finish" to this task
      doTheWork();
    }

    if (exception != null) {
      throw exception;
    }

    return getConfiguration().getString(RESULT_KEY);
  }

  @Override
  public String getMessage() {
    return "Message is:" + getConfiguration().getString(RESULT_KEY);
  }

  protected void doTheWork() throws Exception {
    Thread.sleep(sleepMillis()); // kinda working
  }

  protected long sleepMillis() {
    return getConfiguration().getLong(SLEEP_MILLIS_KEY, 10L);
  }
}
