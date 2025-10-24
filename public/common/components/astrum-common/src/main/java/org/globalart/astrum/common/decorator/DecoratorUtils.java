
package org.globalart.astrum.common.decorator;

import javax.annotation.Nullable;

/**
 * @since 3.8
 */
public class DecoratorUtils
{
  private DecoratorUtils() {
  }

  /**
   * For a decorated object, look down through the chain of wrapped objects for a specific type of class.
   *
   * For example class C wraps class B which wraps class A. The outer object is class C but some code/logic might need
   * something specific to class A. getDecoratedEntity(c, A.class) would look for and return the A object.
   *
   * @param decoratedObject The decorated object to inspect. This is normally an instance of {@link DecoratedObject}
   *          which provides the {@link DecoratedObject#getWrappedObject()} method.
   *
   *          Note that it is not strongly typed to allow for easier calling since normally the type of
   *          the object you have is an interface. For example DecoratorUtilsTest#DecoratorTest does not
   *          implement DecoratedObject, rather the abstract class DecoratorUtilsTest#DecoratedTest does)
   * @param clazz The class type to search for
   * @return The instance if found, otherwise null
   */
  @Nullable
  @SuppressWarnings("unchecked")
  public static <T> T getDecoratedEntity(final Object decoratedObject, final Class<T> clazz) {
    if (clazz.isAssignableFrom(decoratedObject.getClass())) {
      // the object itself is the one we are looking for
      return (T) decoratedObject;
    }

    if (!(decoratedObject instanceof DecoratedObject)) {
      // not a DecoratedObject, no need to look further
      return null;
    }

    T wrappedObject = ((DecoratedObject<T>) decoratedObject).getWrappedObject();
    if (clazz.isAssignableFrom(wrappedObject.getClass())) {
      // wrapped object is the one we are looking for
      return wrappedObject;
    }

    if (wrappedObject instanceof DecoratedObject) {
      // nest and keep looking
      return getDecoratedEntity(wrappedObject, clazz);
    }

    return null;
  }
}
