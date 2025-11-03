
package org.globalart.astrum.thread;

import java.util.concurrent.Callable;
import java.util.concurrent.ScheduledExecutorService;
import java.util.function.Supplier;

import org.globalart.astrum.security.subject.CurrentSubjectSupplier;
import org.globalart.astrum.thread.internal.MDCAwareCallable;
import org.globalart.astrum.thread.internal.MDCAwareRunnable;

import org.apache.shiro.concurrent.SubjectAwareScheduledExecutorService;
import org.apache.shiro.subject.Subject;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * A modification of Shiro's {@link SubjectAwareScheduledExecutorService} that in turn returns always the same, supplied
 * {@link
 * Subject} to bind threads with.
 *
 * @since 3.31
 */
public class AstrumScheduledExecutorService
    extends SubjectAwareScheduledExecutorService
{
  private final Supplier<Subject> subjectSupplier;

  public AstrumScheduledExecutorService(final ScheduledExecutorService target, final Supplier<Subject> subjectSupplier) {
    super(checkNotNull(target));
    this.subjectSupplier = checkNotNull(subjectSupplier);
  }

  /**
   * Override, use our SubjectProvider to get subject from.
   */
  @Override
  protected Subject getSubject() {
    return subjectSupplier.get();
  }

  @Override
  protected Runnable associateWithSubject(Runnable r) {
    Subject subject = getSubject();
    return subject.associateWith(new MDCAwareRunnable(r));
  }

  @Override
  protected <T> Callable<T> associateWithSubject(Callable<T> task) {
    Subject subject = getSubject();
    return subject.associateWith(new MDCAwareCallable<>(task));
  }

  //
  // Factory access
  //

  public static AstrumScheduledExecutorService forFixedSubject(
      final ScheduledExecutorService target,
      final Subject subject)
  {
    return new AstrumScheduledExecutorService(target, () -> subject);
  }

  public static AstrumScheduledExecutorService forCurrentSubject(final ScheduledExecutorService target) {
    return new AstrumScheduledExecutorService(target, new CurrentSubjectSupplier());
  }
}
