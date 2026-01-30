
package org.globalart.astrum.repository.httpclient;

import java.io.IOException;

import com.google.common.annotations.VisibleForTesting;
import org.apache.http.HttpHost;
import org.apache.http.HttpRequest;
import org.apache.http.client.config.RequestConfig;
import org.apache.http.client.methods.CloseableHttpResponse;
import org.apache.http.client.methods.Configurable;
import org.apache.http.conn.ClientConnectionManager;
import org.apache.http.impl.client.CloseableHttpClient;
import org.apache.http.params.HttpParams;
import org.apache.http.protocol.HttpContext;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Support for wrapping {@link CloseableHttpClient}s.
 *
 * <p>
 * Implements {@link Configurable} so that consumers which unconditionally cast the wrapped
 * client (for example RESTEasy's {@code ManualClosingApacheHttpClient43Engine} in RESTEasy 7+)
 * can retrieve the delegate's {@link RequestConfig} instead of failing with a
 * {@link ClassCastException}.
 */
public abstract class FilteredHttpClientSupport
    extends CloseableHttpClient
    implements Configurable
{
  private final CloseableHttpClient delegate;

  public FilteredHttpClientSupport(final CloseableHttpClient delegate) {
    this.delegate = checkNotNull(delegate);
  }

  @Override
  public HttpParams getParams() {
    return delegate.getParams();
  }

  @Override
  public ClientConnectionManager getConnectionManager() {
    return delegate.getConnectionManager();
  }

  @Override
  public RequestConfig getConfig() {
    if (delegate instanceof Configurable) {
      return ((Configurable) delegate).getConfig();
    }
    return RequestConfig.DEFAULT;
  }

  @Override
  protected CloseableHttpResponse doExecute(
      final HttpHost target,
      final HttpRequest request,
      final HttpContext context) throws IOException
  {
    try {
      return filter(target, new Filterable()
      {
        @Override
        public CloseableHttpResponse call() throws IOException {
          return delegate.execute(target, request, context);
        }

        @Override
        public HttpRequest getRequest() {
          return request;
        }

        @Override
        public HttpContext getContext() {
          return context;
        }
      });
    }
    catch (InterruptedException e) {
      Thread.currentThread().interrupt();
      throw new RuntimeException(e);
    }
  }

  @Override
  public void close() throws IOException {
    delegate.close();
  }

  protected abstract CloseableHttpResponse filter(
      final HttpHost target,
      final Filterable filterable) throws IOException, InterruptedException;

  @VisibleForTesting
  public interface Filterable
  {
    CloseableHttpResponse call() throws IOException;

    HttpRequest getRequest();

    HttpContext getContext();
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "delegate=" + delegate +
        '}';
  }
}
