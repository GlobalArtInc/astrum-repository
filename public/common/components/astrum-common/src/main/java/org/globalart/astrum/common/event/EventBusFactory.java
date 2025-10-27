
package org.globalart.astrum.common.event;

import java.lang.reflect.Constructor;
import java.lang.reflect.Method;
import java.util.concurrent.Executor;

import com.google.common.base.Throwables;
import com.google.common.eventbus.EventBus;
import com.google.common.eventbus.SubscriberExceptionHandler;

import static com.google.common.util.concurrent.MoreExecutors.directExecutor;

/**
 * Factory to create custom {@link EventBus} instances with behaviour not exposed via the public API.
 *
 * @since 3.2.1
 */
public class EventBusFactory
{
  private EventBusFactory() {
    // empty
  }

  /**
   * Creates a reentrant {@link EventBus} that dispatches events immediately as they appear on the same thread.
   *
   * (The old Guava behaviour used thread-local queues to provide strong non-reentrant ordering.)
   */
  public static EventBus reentrantEventBus(final String name) {
    return newEventBus(name, directExecutor());
  }

  /**
   * Creates a reentrant {@link EventBus} that dispatches events immediately as they appear using the executor.
   *
   * (The old Guava behaviour used a global queue to provide weak non-reentrant ordering before async dispatch.)
   */
  public static EventBus reentrantAsyncEventBus(final String name, final Executor executor) {
    return newEventBus(name, executor);
  }

  private static EventBus newEventBus(final String name, final Executor executor) {
    try {
      Class<?> dispatcherClass = EventBus.class.getClassLoader().loadClass("com.google.common.eventbus.Dispatcher");

      // immediate dispatcher means events are always processed in a reentrant fashion
      Method immediateDispatcherMethod = dispatcherClass.getDeclaredMethod("immediate");
      immediateDispatcherMethod.setAccessible(true);

      // EventBus constructor that accepts custom executor is not yet part of the public API
      Constructor<EventBus> eventBusConstructor = EventBus.class.getDeclaredConstructor(
          String.class, Executor.class, dispatcherClass, SubscriberExceptionHandler.class);
      eventBusConstructor.setAccessible(true);

      Object immediateDispatcher = immediateDispatcherMethod.invoke(null);
      SubscriberExceptionHandler exceptionHandler = new Slf4jSubscriberExceptionHandler(name);

      return eventBusConstructor.newInstance(name, executor, immediateDispatcher, exceptionHandler);
    }
    catch (Exception e) {
      Throwables.throwIfUnchecked(e);
      throw new LinkageError("Unable to create EventBus with custom executor", e);
    }
  }
}
