
package org.globalart.astrum.test.util;

import java.lang.reflect.Field;
import java.util.Optional;

public final class Whitebox
{
  private Whitebox() {
    // private
  }

  /**
   * Uses reflection to set an intenal value.
   */
  public static void setInternalState(final Object object, final String fieldName, final Object value) {
    try {
      Class<?> clazz = object.getClass();
      Field field = getDeclaredField(clazz, fieldName)
          .orElseGet(() -> getField(clazz, fieldName));
      field.setAccessible(true);
      field.set(object, value);
    }
    catch (IllegalArgumentException | IllegalAccessException | SecurityException e) {
      throw new RuntimeException(e);
    }
  }

  private static Optional<Field> getDeclaredField(final Class<?> clazz, final String fieldName) {
    try {
      return Optional.of(clazz.getDeclaredField(fieldName));
    }
    catch (NoSuchFieldException | SecurityException e) {
      return Optional.empty();
    }
  }

  private static Field getField(final Class<?> clazz, final String fieldName) {
    try {
      return clazz.getField(fieldName);
    }
    catch (NoSuchFieldException | SecurityException e) {
      throw new RuntimeException(e);
    }
  }
}
