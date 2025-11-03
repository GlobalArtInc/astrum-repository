
package org.globalart.astrum.common.lifecycle;

import org.globalart.astrum.common.lifecycle.LifecycleSupport.State;

import org.junit.Test;

import static org.junit.Assert.assertTrue;
import static org.junit.Assert.fail;

/**
 * Tests for {@link LifecycleSupport}.
 */
public class LifecycleSupportTest
{
  private static void assertState(final LifecycleSupport lifecycle, final State state) {
    assertTrue(lifecycle.is(state));
    final boolean[] resultHolder = new boolean[1];
    Thread asyncAssert = new Thread(new Runnable()
    {
      @Override
      public void run() {
        resultHolder[0] = lifecycle.is(state);
      }
    });
    try {
      asyncAssert.start();
      asyncAssert.join(2_000);
      assertTrue(resultHolder[0]);
    }
    catch (Exception e) {
      fail("Unable to check state asynchronously" + e);
    }
  }

  @Test
  public void startStopStartStop() throws Exception {
    LifecycleSupport underTest = new LifecycleSupport();

    assertState(underTest, State.NEW);

    underTest.start();
    assertState(underTest, State.STARTED);

    underTest.stop();
    assertState(underTest, State.STOPPED);

    underTest.start();
    assertState(underTest, State.STARTED);

    underTest.stop();
    assertState(underTest, State.STOPPED);
  }

  @Test
  public void stopBeforeStartDisallowed() throws Exception {
    LifecycleSupport underTest = new LifecycleSupport();

    assertState(underTest, State.NEW);

    try {
      underTest.stop();
    }
    catch (IllegalStateException e) {
      // expected
    }

    assertState(underTest, State.NEW);
  }

  @Test
  public void startAfterStartDisallowed() throws Exception {
    LifecycleSupport underTest = new LifecycleSupport();

    assertState(underTest, State.NEW);

    underTest.start();

    assertState(underTest, State.STARTED);

    try {
      underTest.start();
    }
    catch (IllegalStateException e) {
      // expected
    }

    assertState(underTest, State.STARTED);
  }

  @Test
  public void stopAfterStopDisallowed() throws Exception {
    LifecycleSupport underTest = new LifecycleSupport();

    assertState(underTest, State.NEW);

    underTest.start();

    assertState(underTest, State.STARTED);

    underTest.stop();

    assertState(underTest, State.STOPPED);

    try {
      underTest.stop();
    }
    catch (IllegalStateException e) {
      // expected
    }

    assertState(underTest, State.STOPPED);
  }

  @Test
  public void startAfterFailureDisallowed() throws Exception {
    LifecycleSupport underTest = new LifecycleSupport()
    {
      @Override
      protected void doStart() throws Exception {
        throw new TestException();
      }
    };

    assertState(underTest, State.NEW);

    try {
      underTest.start();
    }
    catch (TestException e) {
      // expected
    }

    assertState(underTest, State.FAILED);

    try {
      underTest.start();
    }
    catch (IllegalStateException e) {
      // expected
    }

    assertState(underTest, State.FAILED);
  }

  @Test
  public void stopAfterFailureDisallowed() throws Exception {
    LifecycleSupport underTest = new LifecycleSupport()
    {
      @Override
      protected void doStart() throws Exception {
        throw new TestException();
      }
    };

    assertState(underTest, State.NEW);

    try {
      underTest.start();
    }
    catch (TestException e) {
      // expected
    }

    assertState(underTest, State.FAILED);

    try {
      underTest.stop();
    }
    catch (IllegalStateException e) {
      // expected
    }

    assertState(underTest, State.FAILED);
  }

  @Test
  public void startException() throws Exception {
    LifecycleSupport underTest = new LifecycleSupport()
    {
      @Override
      protected void doStart() throws Exception {
        throw new TestException();
      }
    };

    try {
      underTest.start();
      fail();
    }
    catch (TestException e) {
      // expected
    }

    assertState(underTest, State.FAILED);
  }

  @Test
  public void startError() throws Exception {
    LifecycleSupport underTest = new LifecycleSupport()
    {
      @Override
      protected void doStart() throws Exception {
        throw new TestError();
      }
    };

    try {
      underTest.start();
      fail();
    }
    catch (TestError e) {
      // expected
    }

    assertState(underTest, State.FAILED);
  }

  @Test
  public void stopException() throws Exception {
    LifecycleSupport underTest = new LifecycleSupport()
    {
      @Override
      protected void doStop() throws Exception {
        throw new TestException();
      }
    };

    underTest.start();
    try {
      underTest.stop();
      fail();
    }
    catch (TestException e) {
      // expected
    }

    assertState(underTest, State.FAILED);
  }

  @Test
  public void stopError() throws Exception {
    LifecycleSupport underTest = new LifecycleSupport()
    {
      @Override
      protected void doStop() throws Exception {
        throw new TestError();
      }
    };

    underTest.start();
    try {
      underTest.stop();
      fail();
    }
    catch (TestError e) {
      // expected
    }

    assertState(underTest, State.FAILED);
  }
}
