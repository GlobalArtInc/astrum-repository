
package org.globalart.astrum.repository.cache.internal;

import java.util.concurrent.TimeUnit;
import javax.annotation.Nullable;
import javax.cache.Cache;
import javax.cache.Cache.Entry;
import javax.cache.expiry.CreatedExpiryPolicy;
import javax.cache.expiry.Duration;
import jakarta.validation.Valid;
import jakarta.validation.constraints.Min;
import jakarta.validation.constraints.NotNull;

import org.globalart.astrum.common.time.Time;
import org.globalart.astrum.cache.CacheHelper;
import org.globalart.astrum.common.stateguard.Guarded;
import org.globalart.astrum.httpclient.config.AuthenticationConfiguration;
import org.globalart.astrum.httpclient.config.UsernameAuthenticationConfiguration;
import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.cache.NegativeCacheFacet;
import org.globalart.astrum.repository.cache.NegativeCacheKey;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.config.ConfigurationFacet;
import org.globalart.astrum.repository.httpclient.HttpClientConfig;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Status;

import com.google.common.annotations.VisibleForTesting;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.FacetSupport.State.STARTED;

/**
 * Default {@link NegativeCacheFacet} implementation.
 *
 * @since 3.0
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class NegativeCacheFacetImpl
    extends FacetSupport
    implements NegativeCacheFacet
{
  private final CacheHelper cacheHelper;

  @VisibleForTesting
  static final String CONFIG_KEY = "negativeCache";

  @VisibleForTesting
  static class Config
  {
    @Valid
    @Nullable
    public AuthenticationConfiguration authentication;

    @NotNull
    public Boolean enabled = Boolean.TRUE;

    /**
     * Time-to-live minutes.
     */
    @NotNull
    @Min(0)
    public Integer timeToLive = Time.hours(24).toMinutesI();

    @Override
    public String toString() {
      return getClass().getSimpleName() + "{" +
          "enabled=" + enabled +
          ", timeToLive=" + timeToLive +
          '}';
    }
  }

  private Config config;

  private HttpClientConfig httpConfig;

  private Cache<NegativeCacheKey, Status> cache;

  @Autowired
  public NegativeCacheFacetImpl(final CacheHelper cacheHelper) {
    this.cacheHelper = checkNotNull(cacheHelper);
  }

  @Override
  protected void doValidate(final Configuration configuration) throws Exception {
    facet(ConfigurationFacet.class).validateSection(configuration, CONFIG_KEY, Config.class);
  }

  @Override
  protected void doConfigure(final Configuration configuration) throws Exception {
    config = facet(ConfigurationFacet.class).readSection(configuration, CONFIG_KEY, Config.class);
    log.debug("Config: {}", config);
  }

  @Override
  protected void doInit(final Configuration configuration) throws Exception {
    super.doInit(configuration);

    // create cache if enabled
    if (config.enabled) {
      maybeCreateCache();
    }
  }

  @Override
  protected void doUpdate(final Configuration configuration) throws Exception {
    Config previous = config;
    super.doUpdate(configuration);

    // re-create cache if enabled or cache settings changed, or when the http config authentication is used
    if (config.enabled) {
      httpConfig = facet(ConfigurationFacet.class).readSection(configuration, "httpclient", HttpClientConfig.class);
      if (!previous.enabled || !config.timeToLive.equals(previous.timeToLive)
          || httpConfig.authentication instanceof UsernameAuthenticationConfiguration) {
        maybeDestroyCache();
        maybeCreateCache();
      }
    }
    else {
      // else destroy cache if disabled
      maybeDestroyCache();
    }
  }

  @Override
  protected void doDelete() throws Exception {
    maybeDestroyCache();
    config = null;
  }

  @Override
  protected void doDestroy() throws Exception {
    cache = null;
    config = null;
  }

  private void maybeCreateCache() {
    if (cache == null) {
      log.debug("Creating negative-cache for: {}", getRepository());
      cache = cacheHelper.maybeCreateCache(getCacheName(), NegativeCacheKey.class, Status.class,
          CreatedExpiryPolicy.factoryOf(new Duration(TimeUnit.MINUTES, config.timeToLive)));
      log.debug("Created negative-cache: {}", cache);
    }
  }

  private void maybeDestroyCache() {
    log.debug("Destroying negative-cache for: {}", getRepository());
    cacheHelper.maybeDestroyCache(getCacheName());
    cache = null;
  }

  @Override
  @Guarded(by = STARTED)
  public Status get(final NegativeCacheKey key) {
    checkNotNull(key);
    if (cache != null) {
      return cache.get(key);
    }
    return null;
  }

  @Override
  @Guarded(by = STARTED)
  public void put(final NegativeCacheKey key, final Status status) {
    checkNotNull(key);
    checkNotNull(status);
    if (cache != null) {
      log.debug("Adding {}={} to negative-cache of {}", key, status, getRepository());
      cache.put(key, status);
    }
  }

  @Override
  @Guarded(by = STARTED)
  public void invalidate(final NegativeCacheKey key) {
    checkNotNull(key);
    if (cache != null && cache.remove(key)) {
      log.debug("Removing {} from negative-cache of {}", key, getRepository());
    }
  }

  @Override
  public void invalidateSubset(final NegativeCacheKey key) {
    if (cache != null) {
      invalidate(key);
      for (final Entry<NegativeCacheKey, Status> entry : cache) {
        if (!key.equals(entry.getKey()) && key.isParentOf(entry.getKey())) {
          invalidate(entry.getKey());
        }
      }
    }
  }

  @Override
  @Guarded(by = STARTED)
  public void invalidate() {
    if (cache != null) {
      log.debug("Removing all from negative-cache of {}", getRepository());
      cache.removeAll();
    }
  }

  @Override
  public NegativeCacheKey getCacheKey(final Context context) {
    return new PathNegativeCacheKey(context.getRequest().getPath());
  }

  public String getCacheName() {
    return getRepository().getName() + "#negative-cache";
  }
}
