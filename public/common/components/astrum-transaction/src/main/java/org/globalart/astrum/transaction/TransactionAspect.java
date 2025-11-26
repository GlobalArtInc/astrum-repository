
package org.globalart.astrum.transaction;

import org.aspectj.lang.ProceedingJoinPoint;
import org.aspectj.lang.annotation.Around;
import org.aspectj.lang.annotation.Aspect;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static org.globalart.astrum.transaction.UnitOfWork.openSession;
import static org.globalart.astrum.transaction.UnitOfWork.peekTransaction;

@Aspect
public class TransactionAspect
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Around("@annotation(transactional) && execution(* *(..))")
  public Object invoke(final ProceedingJoinPoint mi, final Transactional transactional) throws Throwable {
    log.trace("Invoking: {}", mi);
    TransactionalStore<?> store = null;
    if (mi.getTarget() instanceof TransactionalStore<?>) {
      store = (TransactionalStore<?>) mi.getTarget();
    }

    Transaction tx = peekTransaction();
    if (tx != null) { // nested transactional session
      if (store != null) {
        tx.capture(store);
      }
      if (tx.isActive()) {
        return mi.proceed(); // no need to wrap active transaction
      }
      return proceedWithTransaction(mi, transactional, tx);
    }

    try (TransactionalSession<?> session = openSession(store, transactional.isolation())) {
      return proceedWithTransaction(mi, transactional, session.getTransaction());
    }
  }

  public static Object proceedWithTransaction(
      final ProceedingJoinPoint aspect,
      final Transactional spec,
      final Transaction tx) throws Throwable
  {
    return new TransactionalWrapper(spec, aspect).proceedWithTransaction(tx);
  }
}
