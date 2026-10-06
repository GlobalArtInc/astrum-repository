
package org.globalart.astrum.app;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import jakarta.inject.Provider;
import jakarta.servlet.http.HttpServletRequest;

import org.globalart.astrum.common.app.AbstractBaseUrlManager;
import org.globalart.astrum.common.app.BaseUrlManager;

import com.google.common.base.Strings;
import org.springframework.beans.factory.annotation.Value;

import org.springframework.stereotype.Component;

/**
 * Default {@link BaseUrlManager}.
 *
 * @since 3.0
 */
@Component
public class BaseUrlManagerImpl
    extends AbstractBaseUrlManager
{
  private volatile boolean force;

  @Autowired
  public BaseUrlManagerImpl(
      final Provider<HttpServletRequest> requestProvider,
      @Value("${org.globalart.astrum.internal.app.BaseUrlManagerImpl.force:false}") final boolean force)
  {
    super(requestProvider);
    this.force = force;
  }

  @Override
  public void setUrl(final String url) {
    this.url = url;
  }

  @Override
  public String getUrl() {
    return url;
  }

  @Override
  public boolean isForce() {
    return force;
  }

  @Override
  public void setForce(final boolean force) {
    this.force = force;
  }

  /**
   * Detect base-url from forced settings, request or non-forced settings.
   */
  @Nullable
  @Override
  public String detectUrl() {
    // force base-url always wins if set
    if (force && !Strings.isNullOrEmpty(url)) {
      return url;
    }
    return super.detectUrl();
  }
}
