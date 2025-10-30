
package org.globalart.astrum.common.thread;

import java.lang.reflect.InvocationHandler;
import java.lang.reflect.Proxy;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Helper to create wrappers around components to ensure that the TCCL is properly configured.
 *
 * @since 3.0
 */
public class TcclWrapper
{
  private TcclWrapper() {
    // empty
  }

  /**
   * Creates a dynamic-proxy for type, delegating to target and setting the TCCL to class-loader before invocation.
   */
  @SuppressWarnings("unchecked")
  public static <T> T create(final Class<T> type, final T target, final ClassLoader classLoader) {
    checkNotNull(type);
    checkNotNull(target);
    checkNotNull(classLoader);

    InvocationHandler handler = (proxy, method, args) -> {
      try (TcclBlock tccl = TcclBlock.begin(classLoader)) {
        return method.invoke(target, args);
      }
    };

    return (T) Proxy.newProxyInstance(type.getClassLoader(), new Class[]{type}, handler);
  }
}
