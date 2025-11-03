
package org.globalart.astrum.thread;

import java.util.concurrent.Callable;
import java.util.concurrent.ExecutorService;
import java.util.function.Supplier;

import org.globalart.astrum.security.subject.CurrentSubjectSupplier;
import org.globalart.astrum.thread.internal.MDCAwareCallable;
import org.globalart.astrum.thread.internal.MDCAwareRunnable;

import org.apache.shiro.concurrent.SubjectAwareExecutorService;
import org.apache.shiro.subject.Subject;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * A modification of Shiro's {@link SubjectAwareExecutorService} that in turn returns
 * always the same, supplied {@link Subject} to bind threads with.
 *
 * @since 2.6
 */
public class AstrumExecutorService
    extends SubjectAwareExecutorService
{
  private final Supplier<Subject> subjectSupplier;

  public AstrumExecutorService(final ExecutorService target, final Supplier<Subject> subjectSupplier) {
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

  public static AstrumExecutorService forFixedSubject(final ExecutorService target, final Subject subject) {
    return new AstrumExecutorService(target, () -> subject);
  }

  public static AstrumExecutorService forCurrentSubject(final ExecutorService target) {
    return new AstrumExecutorService(target, new CurrentSubjectSupplier());
  }
}
