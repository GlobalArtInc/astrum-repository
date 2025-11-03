
package org.globalart.astrum.common.log;

import java.util.concurrent.TimeUnit;
import java.util.function.BiConsumer;
import java.util.function.BiPredicate;

import org.junit.Test;
import org.mockito.InOrder;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;
import org.slf4j.Logger;

import static org.mockito.Mockito.inOrder;
import static org.globalart.astrum.common.log.ExceptionSummarizer.sameText;
import static org.globalart.astrum.common.log.ExceptionSummarizer.sameType;
import static org.globalart.astrum.common.log.ExceptionSummarizer.warn;

/**
 * Tests for {@link ExceptionSummarizer}.
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class ExceptionSummarizerTest

{
  @Mock
  private Logger log;

  private Exception firstCause = new IllegalArgumentException("first");

  private Exception secondCause = new IllegalStateException("second");

  private Exception thirdCause = new IllegalStateException("third");

  private Exception repeatCause = new IllegalStateException();

  private TestExceptionSummarizer underTest;

  @Test
  public void summarizeExceptionsByType() throws Exception {
    underTest = new TestExceptionSummarizer(sameType(), warn(log));

    underTest.log("oops", firstCause); // <-- full stack

    underTest.log("oops", secondCause); // <-- full stack, because type changed

    underTest.log("oops", thirdCause);
    underTest.log("oops", repeatCause);
    underTest.sleep(5, TimeUnit.SECONDS);
    underTest.log("oops", repeatCause); // <-- summary (3 repeats)

    underTest.log("oops", repeatCause);
    underTest.log("oops", repeatCause);
    underTest.log("oops", repeatCause);
    underTest.log("oops", repeatCause);
    underTest.sleep(10, TimeUnit.SECONDS);
    underTest.log("oops", repeatCause); // <-- summary (5 repeats)

    underTest.log("oops", repeatCause);
    underTest.log("oops", repeatCause);
    underTest.sleep(60, TimeUnit.SECONDS);
    underTest.log("oops", repeatCause); // <-- full stack, because over 1min since last summary

    underTest.sleep(5, TimeUnit.SECONDS);
    underTest.log("oops", repeatCause); // <-- summary (1 repeat)

    InOrder inOrder = inOrder(log);
    inOrder.verify(log).warn("oops", firstCause);
    inOrder.verify(log).warn("oops", secondCause);
    inOrder.verify(log)
        .warn("oops: java.lang.IllegalStateException - occurred 3 times in last 5 seconds", (Exception) null);
    inOrder.verify(log)
        .warn("oops: java.lang.IllegalStateException - occurred 5 times in last 10 seconds", (Exception) null);
    inOrder.verify(log).warn("oops", repeatCause);
    inOrder.verify(log)
        .warn("oops: java.lang.IllegalStateException - occurred 1 times in last 5 seconds", (Exception) null);
    inOrder.verifyNoMoreInteractions();
  }

  @Test
  public void summarizeExceptionsByText() throws Exception {
    underTest = new TestExceptionSummarizer(sameText(), warn(log));

    underTest.log("oops", firstCause); // <-- full stack

    underTest.log("oops", secondCause); // <-- full stack, because type changed

    underTest.log("oops", thirdCause); // <-- full stack, because text changed

    underTest.log("oops", repeatCause); // <-- full stack, because text changed

    underTest.sleep(5, TimeUnit.SECONDS);
    underTest.log("oops", repeatCause); // <-- summary (1 repeat)

    underTest.log("oops", repeatCause);
    underTest.log("oops", repeatCause);
    underTest.log("oops", repeatCause);
    underTest.log("oops", repeatCause);
    underTest.sleep(10, TimeUnit.SECONDS);
    underTest.log("oops", repeatCause); // <-- summary (5 repeats)

    underTest.log("oops", repeatCause);
    underTest.log("oops", repeatCause);
    underTest.sleep(60, TimeUnit.SECONDS);
    underTest.log("oops", repeatCause); // <-- full stack, because over 1min since last summary

    underTest.sleep(5, TimeUnit.SECONDS);
    underTest.log("oops", repeatCause); // <-- summary (1 repeat)

    InOrder inOrder = inOrder(log);
    inOrder.verify(log).warn("oops", firstCause);
    inOrder.verify(log).warn("oops", secondCause);
    inOrder.verify(log).warn("oops", thirdCause);
    inOrder.verify(log).warn("oops", repeatCause);
    inOrder.verify(log)
        .warn("oops: java.lang.IllegalStateException - occurred 1 times in last 5 seconds", (Exception) null);
    inOrder.verify(log)
        .warn("oops: java.lang.IllegalStateException - occurred 5 times in last 10 seconds", (Exception) null);
    inOrder.verify(log).warn("oops", repeatCause);
    inOrder.verify(log)
        .warn("oops: java.lang.IllegalStateException - occurred 1 times in last 5 seconds", (Exception) null);
    inOrder.verifyNoMoreInteractions();
  }

  /**
   * Stubbed {@link ExceptionSummarizer} that lets tests move time forward without sleeping.
   */
  private static class TestExceptionSummarizer
      extends ExceptionSummarizer
  {
    private long currentTimeMillis = System.currentTimeMillis();

    TestExceptionSummarizer(
        final BiPredicate<Exception, Exception> matcher,
        final BiConsumer<String, Exception> logger)
    {
      super(matcher, logger);
    }

    public void sleep(final int duration, final TimeUnit unit) {
      this.currentTimeMillis += unit.toMillis(duration);
    }

    @Override
    long currentTimeMillis() {
      return currentTimeMillis;
    }
  }
}
