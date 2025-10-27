
package org.globalart.astrum.common.io;

import java.io.IOException;
import java.io.InputStream;
import java.io.ObjectInputFilter;
import java.io.ObjectInputStream;
import java.io.ObjectStreamClass;
import java.util.function.Predicate;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Allows a custom class loader to be used with ObjectInputStream
 *
 * @since 3.6
 */
public class ObjectInputStreamWithClassLoader
    extends ObjectInputStream
{
  private static final Logger log = LoggerFactory.getLogger(ObjectInputStreamWithClassLoader.class);

  @FunctionalInterface
  public interface LoadingFunction
  {
    Class<?> loadClass(String name) throws ClassNotFoundException;
  }

  private final LoadingFunction classLoading;

  /**
   * Creates an ObjectInputStream with a custom class loading function and a class filter.
   *
   * @param inputStream the input stream containing serialized data
   * @param classLoading the function to load classes by name
   * @param classFilter a predicate that returns true for allowed class names
   * @throws IOException if an I/O error occurs reading the stream header
   * @since 3.89
   */
  public ObjectInputStreamWithClassLoader(
      final InputStream inputStream,
      final LoadingFunction classLoading,
      final Predicate<Class<?>> classFilter) throws IOException
  {
    super(inputStream);
    this.classLoading = checkNotNull(classLoading);
    setObjectInputFilter(createObjectInputFilter(checkNotNull(classFilter)));
  }

  /**
   * Creates an ObjectInputStream with a custom class loader and a class filter.
   *
   * @param inputStream the input stream containing serialized data
   * @param loader the class loader to use for resolving classes
   * @param classFilter a predicate that returns true for allowed class names
   * @throws IOException if an I/O error occurs reading the stream header
   * @since 3.89
   */
  public ObjectInputStreamWithClassLoader(
      final InputStream inputStream,
      final ClassLoader loader,
      final Predicate<Class<?>> classFilter) throws IOException
  {
    super(inputStream);
    checkNotNull(loader);
    this.classLoading = name -> Class.forName(name, false, loader);
    setObjectInputFilter(createObjectInputFilter(checkNotNull(classFilter)));
  }

  private static ObjectInputFilter createObjectInputFilter(final Predicate<Class<?>> classFilter) {
    return info -> {
      Class<?> serialClass = info.serialClass();
      if (serialClass == null) {
        return ObjectInputFilter.Status.UNDECIDED;
      }

      if (classFilter.test(serialClass)) {
        return ObjectInputFilter.Status.ALLOWED;
      }

      // Allow primitive arrays (e.g., byte[], int[])
      if (serialClass.isArray() && serialClass.getComponentType().isPrimitive()) {
        return ObjectInputFilter.Status.ALLOWED;
      }

      log.warn("Rejecting attempt to deserialize {}", serialClass);

      return ObjectInputFilter.Status.REJECTED;
    };
  }

  @Override
  protected Class<?> resolveClass(final ObjectStreamClass classDesc) throws ClassNotFoundException {
    return classLoading.loadClass(classDesc.getName());
  }
}
