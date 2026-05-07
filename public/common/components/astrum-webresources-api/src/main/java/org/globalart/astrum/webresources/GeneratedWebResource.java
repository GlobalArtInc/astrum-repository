
package org.globalart.astrum.webresources;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;

import org.globalart.astrum.webresources.WebResource.Prepareable;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkState;

/**
 * Support for generated {@link WebResource} implementations.
 *
 * @since 2.8
 */
public abstract class GeneratedWebResource
    implements WebResource, Prepareable
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Override
  public boolean isCacheable() {
    return false;
  }

  @Override
  public long getLastModified() {
    return System.currentTimeMillis();
  }

  @Override
  public long getSize() {
    throw new UnsupportedOperationException("Preparation required");
  }

  @Override
  public InputStream getInputStream() throws IOException {
    throw new UnsupportedOperationException("Preparation required");
  }

  @Override
  public WebResource prepare() throws IOException {
    return new DelegatingWebResource(this)
    {
      private final byte[] content;

      {
        content = generate();
        checkState(content != null);
        log.trace("Generated: {}, {} bytes", getPath(), content.length);
      }

      @Override
      public long getSize() {
        return content.length;
      }

      @Override
      public InputStream getInputStream() throws IOException {
        return new ByteArrayInputStream(content);
      }
    };
  }

  protected abstract byte[] generate() throws IOException;
}
