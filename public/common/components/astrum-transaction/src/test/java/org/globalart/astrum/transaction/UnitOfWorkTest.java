
package org.globalart.astrum.transaction;

import org.globalart.astrum.transaction.ExampleMethods.ExampleNestedStore;

import com.google.common.base.Suppliers;
import org.junit.Test;

/**
 * Test unit-of-work behaviour.
 */
public class UnitOfWorkTest

{
  @Test(expected = NullPointerException.class)
  public void testCannotBeginNullWork() {
    UnitOfWork.begin(null);
  }

  @Test(expected = IllegalStateException.class)
  public void testCannotEndNoWork() {
    UnitOfWork.end();
  }

  @SuppressWarnings("java:S2699") // sonar expects assertions, but best to let this exception bubble up
  @Test
  public void testCanPauseNoWork() {
    UnitOfWork.resume(UnitOfWork.pause());
  }

  @Test(expected = IllegalStateException.class)
  public void testCannotResumeTwice() {
    UnitOfWork.begin(Suppliers.<TransactionalSession<Transaction>>ofInstance(null));
    try {
      UnitOfWork work = UnitOfWork.pause();
      UnitOfWork.resume(work);
      UnitOfWork.resume(work);
    }
    finally {
      UnitOfWork.end();
    }
  }

  @Test(expected = IllegalStateException.class)
  public void testCannotStartTransactionWithNoWork() {
    new ExampleMethods(new ExampleNestedStore()).transactional();
  }
}
