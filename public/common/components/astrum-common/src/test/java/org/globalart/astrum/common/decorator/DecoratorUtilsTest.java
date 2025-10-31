
package org.globalart.astrum.common.decorator;

import org.junit.Test;

import static org.hamcrest.CoreMatchers.equalTo;
import static org.hamcrest.CoreMatchers.notNullValue;
import static org.hamcrest.core.IsNull.nullValue;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.globalart.astrum.common.decorator.DecoratorUtils.getDecoratedEntity;

public class DecoratorUtilsTest
{
  @Test
  public void testGetDecoratedEntity() {
    DecoratorTest first = new DefaultDecoratorTest();
    DecoratorTest second = new TestDecorator1(first);
    DecoratorTest third = new TestDecorator2(second);

    assertThat(getDecoratedEntity(first, DefaultDecoratorTest.class), notNullValue());
    assertThat(getDecoratedEntity(first, TestDecorator1.class), nullValue());
    assertThat(getDecoratedEntity(first, TestDecorator2.class), nullValue());

    assertThat(getDecoratedEntity(second, DefaultDecoratorTest.class), notNullValue());
    assertThat(getDecoratedEntity(second, TestDecorator1.class), notNullValue());
    assertThat(getDecoratedEntity(second, TestDecorator1.class).specialMethod1(), equalTo("special1"));
    assertThat(getDecoratedEntity(second, TestDecorator2.class), nullValue());

    assertThat(getDecoratedEntity(third, DefaultDecoratorTest.class), notNullValue());
    assertThat(getDecoratedEntity(third, TestDecorator1.class), notNullValue());
    assertThat(getDecoratedEntity(second, TestDecorator1.class).specialMethod1(), equalTo("special1"));
    assertThat(getDecoratedEntity(third, TestDecorator2.class), notNullValue());
    assertThat(getDecoratedEntity(third, TestDecorator2.class).specialMethod2(), equalTo("special2"));
  }

  // DECORATOR CLASSES

  private interface DecoratorTest
  {
    String getValue();
  }

  private class DefaultDecoratorTest
      implements DecoratorTest
  {
    @Override
    public String getValue() {
      return "default";
    }
  }

  private abstract class DecoratedTest
      implements DecoratorTest, DecoratedObject<DecoratorTest>
  {
    private final DecoratorTest wrapped;

    DecoratedTest(final DecoratorTest wrapped) {
      this.wrapped = wrapped;
    }

    @Override
    public String getValue() {
      return wrapped.getValue();
    }

    @Override
    public DecoratorTest getWrappedObject() {
      return wrapped;
    }
  }

  private final class TestDecorator1
      extends DecoratedTest
  {
    TestDecorator1(final DecoratorTest wrapped) {
      super(wrapped);
    }

    @Override
    public String getValue() {
      return "overridden";
    }

    String specialMethod1() {
      return "special1";
    }
  }

  private final class TestDecorator2
      extends DecoratedTest
  {
    TestDecorator2(final DecoratorTest wrapped) {
      super(wrapped);
    }

    String specialMethod2() {
      return "special2";
    }
  }
}
