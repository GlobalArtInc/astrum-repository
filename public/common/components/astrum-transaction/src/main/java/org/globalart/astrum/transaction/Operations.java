
package org.globalart.astrum.transaction;

import java.lang.annotation.Annotation;
import java.util.function.Supplier;

import javax.annotation.Nullable;

import com.google.common.annotations.VisibleForTesting;
import com.google.common.base.Throwables;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.transaction.TransactionIsolation.STANDARD;
import static org.globalart.astrum.transaction.Transactional.DEFAULT_REASON;
import static org.globalart.astrum.transaction.UnitOfWork.openSession;
import static org.globalart.astrum.transaction.UnitOfWork.peekTransaction;

/**
 * Fluent API for wrapping lambda operations with {@link Transactional} behaviour:
 *
 * <pre>
 * Value value = Transactional.operation.retryOn(IOException.class).call(() -> {
 *   // do transactional work which returns a value
 * });
 *
 * Transactional.operation.retryOn(IOException.class).run(() -> {
 *   // do transactional work which doesn't return a value
 * });
 * </pre>
 *
 * You can choose to supply your own transactions instead of relying on {@link UnitOfWork}:
 *
 * <pre>
 * Transactional.operation.withDb(myTxSupplier).retryOn(IOException.class).call(() -> {
 *   // do transactional work
 * });
 * </pre>
 *
 * If your lambda throws a checked exception then you need to explicitly declare this:
 *
 * <pre>
 * Transactional.operation.throwing(PersistenceException.class).call(() -> {
 *   // do transactional work
 * });
 * </pre>
 *
 * You can also use stereotype annotations meta-annotated with {@link Transactional}:
 *
 * <pre>
 * Transactional.operation.stereotype(TransactionalGet.class).call(() -> {
 *   // do transactional work
 * });
 * </pre>
 *
 * The result of each fluent step (except call and run) can be cached and re-used.
 *
 * @since 3.0
 */
@SuppressWarnings("unchecked")
public class Operations<E extends Exception, B extends Operations<E, B>>
{
  private static final Logger log = LoggerFactory.getLogger(Operations.class);

  private static final Class<?>[] NOTHING = {};

  @VisibleForTesting
  static final Transactional DEFAULT_SPEC = new TransactionalImpl(DEFAULT_REASON, NOTHING, NOTHING, NOTHING, STANDARD);

  @VisibleForTesting
  final Transactional spec;

  @Nullable
  private final Class<E> throwing;

  @Nullable
  private final TransactionalStore<?> store;

  /**
   * @see Transactional#reason()
   * @since 3.20
   */
  public final B reason(final String reason) {
    return (B) copy(new TransactionalImpl(reason, spec.commitOn(), spec.retryOn(), spec.swallow(), spec.isolation()),
        throwing, store);
  }

  /**
   * @see Transactional#commitOn()
   */
  @SafeVarargs
  public final B commitOn(final Class<? extends Exception>... exceptionTypes) {
    Class<?>[] commitOn = deepCheckNotNull(exceptionTypes).clone();
    return (B) copy(new TransactionalImpl(spec.reason(), commitOn, spec.retryOn(), spec.swallow(), spec.isolation()),
        throwing, store);
  }

  /**
   * @see Transactional#retryOn()
   */
  @SafeVarargs
  public final B retryOn(final Class<? extends Exception>... exceptionTypes) {
    Class<?>[] retryOn = deepCheckNotNull(exceptionTypes).clone();
    return (B) copy(new TransactionalImpl(spec.reason(), spec.commitOn(), retryOn, spec.swallow(), spec.isolation()),
        throwing, store);
  }

  /**
   * @see Transactional#swallow()
   */
  @SafeVarargs
  public final B swallow(final Class<? extends Exception>... exceptionTypes) {
    Class<?>[] swallow = deepCheckNotNull(exceptionTypes).clone();
    return (B) copy(new TransactionalImpl(spec.reason(), spec.commitOn(), spec.retryOn(), swallow, spec.isolation()),
        throwing, store);
  }

  /**
   * Applies the given stereotype annotation (meta-annotated with &#064;{@link Transactional}).
   *
   * @since 3.2
   */
  public final B stereotype(final Class<? extends Annotation> annotationType) {
    Transactional metaSpec = annotationType.getAnnotation(Transactional.class);
    checkArgument(metaSpec != null, "Stereotype annotation is not meta-annotated with @Transactional");
    return (B) copy(metaSpec, throwing, store);
  }

  /**
   * Assumes the lambda may throw the given checked exception.
   */
  public <X extends Exception> Operations<X, ?> throwing(final Class<X> exceptionType) {
    return copy(spec, checkNotNull(exceptionType), store);
  }

  /**
   * Uses the given supplier to acquire {@link TransactionalSession}s.
   *
   * @since 3.2
   */
  public final B withDb(final Supplier<? extends TransactionalSession<?>> db) {
    return (B) copy(spec, throwing, db::get);
  }

  /**
   * Uses the given {@link TransactionalStore} to supply {@link TransactionalSession}s.
   *
   * @since 3.19
   */
  public final B withStore(final TransactionalStore<?> _store) {
    return (B) copy(spec, throwing, checkNotNull(_store));
  }

  /**
   * Calls the given operation with {@link Transactional} behaviour.
   */
  public final <T> T call(final Operation<T, E> operation) throws E {
    return transactional(new OperationPoint<>(operation));
  }

  /**
   * Runs the given operation with {@link Transactional} behaviour.
   *
   * @since 3.2
   */
  public final void run(final VoidOperation<E> operation) throws E {
    transactional(new OperationPoint<>(operation));
  }

  /**
   * Default settings.
   */
  protected Operations() {
    this(DEFAULT_SPEC, null, null);
  }

  /**
   * Custom settings.
   */
  protected Operations(
      final Transactional spec,
      @Nullable final Class<E> throwing,
      @Nullable final TransactionalStore<?> store)
  {
    this.spec = checkNotNull(spec);
    this.throwing = throwing;
    this.store = store;
  }

  /**
   * Copies the given settings into a new fluent step.
   */
  protected <X extends Exception> Operations<X, ?> copy(
      final Transactional spec,
      @Nullable final Class<X> throwing,
      @Nullable final TransactionalStore<?> store)
  {
    return new Operations<>(spec, throwing, store);
  }

  /**
   * Invokes the given {@link OperationPoint} using the current settings.
   */
  private <T> T transactional(final OperationPoint<T, E> point) throws E {
    Transaction tx = peekTransaction();
    if (tx != null) { // nested transactional session
      if (store != null) {
        tx.capture(store);
      }
      if (tx.isActive()) {
        return point.proceed(); // no need to wrap active transaction
      }
      return proceedWithTransaction(point, tx);
    }

    try (TransactionalSession<?> session = openSession(store, spec.isolation())) {
      return proceedWithTransaction(point, session.getTransaction());
    }
  }

  private <T> T proceedWithTransaction(final OperationPoint<T, E> point, final Transaction tx) throws E {

    log.trace("Invoking: {} -> {}", spec, point);

    try {
      return (T) new TransactionalWrapper(spec, point).proceedWithTransaction(tx);
    }
    catch (final Throwable e) {
      if (throwing != null) {
        Throwables.propagateIfPossible(e, throwing);
      }
      Throwables.throwIfUnchecked(e);
      throw new RuntimeException(e);
    }
  }

  /**
   * Checks that the given array and its elements are not null.
   */
  private static <T> T[] deepCheckNotNull(final T[] elements) {
    for (T e : elements) {
      checkNotNull(e);
    }
    return elements;
  }
}
