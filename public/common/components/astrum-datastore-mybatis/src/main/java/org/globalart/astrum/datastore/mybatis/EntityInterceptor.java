
package org.globalart.astrum.datastore.mybatis;

import org.apache.ibatis.executor.Executor;
import org.apache.ibatis.plugin.Interceptor;
import org.apache.ibatis.plugin.Invocation;

/**
 * MyBatis {@link Interceptor} that wraps any new {@link Executor} with {@link EntityExecutor}.
 *
 * @since 3.19
 */
final class EntityInterceptor
    implements Interceptor
{
  @Override
  public Object plugin(final Object delegate) {
    if (delegate instanceof Executor) {
      return new EntityExecutor((Executor) delegate);
    }
    return delegate;
  }

  @Override
  public Object intercept(final Invocation invocation) {
    throw new UnsupportedOperationException("unused");
  }
}
