
package org.globalart.astrum.thread.internal;

import java.util.Map;

import org.apache.shiro.util.ThreadContext;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Runnable that properly sets MDC context before invoking the delegate. The delegate will execute in a
 * managed thread with properly set MDC context. To be used with managed threads.
 *
 * Shiro 2.x changed {@link ThreadContext} from an {@code InheritableThreadLocal} to a plain
 * {@code ThreadLocal}, so managed worker threads no longer inherit the submitting thread's
 * SecurityManager/Subject. The submitting thread's ThreadContext resources are captured here (in the
 * constructor) and rebound on the worker thread so the security context propagates to managed threads
 * as it did before the upgrade. The captured context is only applied when the worker thread does not
 * already carry one of its own, and it is removed afterwards so pooled threads are not left bound to a
 * stale context.
 *
 * @since 2.6
 */
public class MDCAwareRunnable
    implements Runnable
{
  private final Runnable delegate;

  private final Map<String, String> mdcContext;

  private final Map<Object, Object> threadContextResources;

  public MDCAwareRunnable(final Runnable delegate) {
    this.delegate = checkNotNull(delegate);
    this.mdcContext = MDCUtils.getCopyOfContextMap();
    // ThreadContext.getResources() returns a defensive copy of the submitting thread's resource map,
    // so this captures a point-in-time snapshot of the submit-time security context: a later mutation
    // on the submitting thread cannot bleed into it before the queued worker runs.
    this.threadContextResources = ThreadContext.getResources();
  }

  @Override
  public void run() {
    // AstrumExecutorService wraps this runnable with subject.associateWith(...), which binds the
    // executor's chosen Subject (e.g. the fixed TASK_SUBJECT used by QuartzThreadPool, StreamCopier and
    // DatabaseStatusDelayedExecutor) on the worker thread before run() executes. Only propagate the
    // captured submitting-thread context when the worker has no Shiro context of its own; replacing an
    // already-bound (fixed) Subject would run the task as the wrong subject and cause authorization and
    // audit regressions, so an existing worker context always wins.
    boolean bindContext = threadContextResources != null && !threadContextResources.isEmpty()
        && ThreadContext.getResources().isEmpty();
    if (bindContext) {
      // Bind the captured security context BEFORE setting the MDC: MDCUtils.setContextMap resolves the
      // current Subject (UserIdMdcHelper), which needs the SecurityManager to be available first.
      ThreadContext.setResources(threadContextResources);
    }
    try {
      MDCUtils.setContextMap(mdcContext);
      delegate.run();
    }
    finally {
      // Only clear what we bound, so a pooled thread is not left holding this task's
      // Subject/SecurityManager. A context the worker already owned is left untouched.
      if (bindContext) {
        ThreadContext.remove();
      }
    }
  }
}
