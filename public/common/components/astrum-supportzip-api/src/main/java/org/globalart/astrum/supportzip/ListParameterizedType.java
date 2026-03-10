
package org.globalart.astrum.supportzip;

import java.lang.reflect.ParameterizedType;
import java.lang.reflect.Type;
import java.util.ArrayList;
import java.util.Objects;

/**
 * Represents a parameterized type for a List&lt;T&gt;.
 *
 * @since 3.29
 */
public class ListParameterizedType
    implements ParameterizedType
{
  private final Type type;

  public ListParameterizedType(final Type type) {
    this.type = type;
  }

  @Override
  public Type[] getActualTypeArguments() {
    return new Type[]{type};
  }

  @Override
  public Type getRawType() {
    return ArrayList.class;
  }

  @Override
  public Type getOwnerType() {
    return null;
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    ListParameterizedType that = (ListParameterizedType) o;
    return Objects.equals(type, that.type);
  }

  @Override
  public int hashCode() {
    return Objects.hash(type);
  }
}
