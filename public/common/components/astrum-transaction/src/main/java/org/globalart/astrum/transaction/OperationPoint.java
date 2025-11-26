
package org.globalart.astrum.transaction;

import java.lang.reflect.Method;
import java.util.function.Supplier;

import com.google.common.base.Suppliers;
import org.aopalliance.intercept.Joinpoint;
import org.aspectj.lang.ProceedingJoinPoint;
import org.aspectj.lang.Signature;
import org.aspectj.lang.reflect.SourceLocation;
import org.aspectj.runtime.internal.AroundClosure;

/**
 * {@link Joinpoint} for operations; handles non-void and void return cases.
 *
 * @since 3.2
 */
final class OperationPoint<T, E extends Exception>
    implements ProceedingJoinPoint
{
  private final Operation<T, E> proceed;

  private final Object self;

  private final Supplier<Method> staticPart;

  /**
   * {@link Joinpoint} for operations which return values.
   */
  OperationPoint(final Operation<T, E> operation) {
    this.proceed = operation;

    // used when logging
    this.self = operation;
    this.staticPart = lazyMethod(operation, "call");
  }

  /**
   * {@link Joinpoint} for operations which don't return values.
   */
  OperationPoint(final VoidOperation<E> operation) {
    // wrap as non-void
    this.proceed = () -> {
      operation.run();
      return null;
    };

    // used when logging
    this.self = operation;
    this.staticPart = lazyMethod(operation, "run");
  }

  @Override
  public T proceed() throws E {
    return proceed.call();
  }

  @Override
  public final Object getThis() {
    return self;
  }

  @Override
  public final StaticPart getStaticPart() {
    throw new UnsupportedOperationException();
  }

  @Override
  public String toString() {
    return self.toString();
  }

  /**
   * 'staticPart' is only used in TransactionalWrapper when trace logging,
   * so we want to defer lookup until it's needed but also cache the result
   */
  private static Supplier<Method> lazyMethod(final Object instance, final String method) {
    return Suppliers.memoize(() -> {
      try {
        return instance.getClass().getMethod(method);
      }
      catch (final NoSuchMethodException e) {
        throw new LinkageError("Missing '" + method + "' method", e);
      }
    });
  }

  @Override
  public String toShortString() {
    throw new UnsupportedOperationException();
  }

  @Override
  public String toLongString() {
    throw new UnsupportedOperationException();
  }

  @Override
  public Object getTarget() {
    return self;
  }

  @Override
  public Object[] getArgs() {
    throw new UnsupportedOperationException();
  }

  @Override
  public Signature getSignature() {
    throw new UnsupportedOperationException();
  }

  @Override
  public SourceLocation getSourceLocation() {
    throw new UnsupportedOperationException();
  }

  @Override
  public String getKind() {
    throw new UnsupportedOperationException();
  }

  @Override
  public void set$AroundClosure(final AroundClosure arc) {
    throw new UnsupportedOperationException();
  }

  @Override
  public Object proceed(final Object[] args) throws Throwable {
    throw new UnsupportedOperationException();
  }
}
