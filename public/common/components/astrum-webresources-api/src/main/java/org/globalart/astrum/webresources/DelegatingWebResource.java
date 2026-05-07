
package org.globalart.astrum.webresources;

import java.io.IOException;
import java.io.InputStream;

import javax.annotation.Nullable;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Delegating {@link WebResource}.
 *
 * @since 2.8
 */
public class DelegatingWebResource
    implements WebResource
{
  private final WebResource delegate;

  public DelegatingWebResource(final WebResource delegate) {
    this.delegate = checkNotNull(delegate);
  }

  @Override
  public String getPath() {
    return delegate.getPath();
  }

  @Override
  @Nullable
  public String getContentType() {
    return delegate.getContentType();
  }

  @Override
  public long getSize() {
    return delegate.getSize();
  }

  @Override
  public long getLastModified() {
    return delegate.getLastModified();
  }

  @Override
  public boolean isCacheable() {
    return delegate.isCacheable();
  }

  @Override
  public InputStream getInputStream() throws IOException {
    return delegate.getInputStream();
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "delegate=" + delegate +
        '}';
  }
}
