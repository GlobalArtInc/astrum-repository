
package org.globalart.astrum.datastore;

import java.util.Map;
import java.util.Map.Entry;

import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;
import org.globalart.astrum.common.stateguard.Transitions;
import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.datastore.api.DataStore;
import org.globalart.astrum.datastore.api.DataStoreConfiguration;

import org.codehaus.plexus.interpolation.EnvarBasedValueSource;
import org.codehaus.plexus.interpolation.InterpolationException;
import org.codehaus.plexus.interpolation.Interpolator;
import org.codehaus.plexus.interpolation.MapBasedValueSource;
import org.codehaus.plexus.interpolation.SingleResponseValueSource;
import org.codehaus.plexus.interpolation.StringSearchInterpolator;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Strings.isNullOrEmpty;
import static java.util.stream.Collectors.toMap;
import static org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport.State.SHUTDOWN;

/**
 * Common support class for {@link DataStore}s.
 *
 * @since 3.19
 */
public abstract class DataStoreSupport<S extends DataSession<?>>
    extends StateGuardLifecycleSupport
    implements DataStore<S>
{
  protected DataStoreConfiguration configuration;

  private String storeName;

  @Override
  public DataStoreConfiguration getConfiguration() {
    return configuration;
  }

  @Override
  public void setConfiguration(final DataStoreConfiguration configuration) {
    checkArgument(!isNullOrEmpty(configuration.getName()));
    this.configuration = configuration;
    this.storeName = configuration.getName();
  }

  @Override
  protected final void doStart() throws Exception {
    doStart(storeName, interpolatedAttributes());
  }

  protected abstract void doStart(final String storeName, final Map<String, String> attributes) throws Exception;

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "configuration=" + configuration +
        '}';
  }

  /**
   * Log the given WARN message along with the store name.
   */
  protected void warn(final String format, final Object... args) {
    log.warn(inStore(format), args);
  }

  /**
   * Log the given INFO message along with the store name.
   */
  protected void info(final String format, final Object... args) {
    log.info(inStore(format), args);
  }

  /**
   * Log the given DEBUG message along with the store name.
   */
  protected void debug(final String format, final Object... args) {
    if (log.isDebugEnabled()) {
      log.debug(inStore(format), args);
    }
  }

  /**
   * Contextualize the given log message with the store name.
   */
  private String inStore(final String message) {
    return storeName + " - " + message;
  }

  /**
   * Interpolate configuration attributes when starting the data store.
   */
  private Map<String, String> interpolatedAttributes() throws Exception {
    Map<String, String> attributes = configuration.getAttributes();

    Interpolator interpolator = new StringSearchInterpolator();
    interpolator.addValueSource(new SingleResponseValueSource("storeName", storeName));
    interpolator.addValueSource(new MapBasedValueSource(attributes));
    interpolator.addValueSource(new MapBasedValueSource(System.getProperties()));
    interpolator.addValueSource(new EnvarBasedValueSource());

    return attributes.entrySet().stream().collect(toMap(Entry::getKey, entry -> {
      try {
        return interpolator.interpolate(entry.getValue());
      }
      catch (InterpolationException e) {
        throw new IllegalArgumentException(e);
      }
    }));
  }

  /**
   * Permanently stops this data store regardless of the current state, disallowing restarts.
   */
  @Override
  @Transitions(to = SHUTDOWN)
  public void shutdown() throws Exception {
    if (isStarted()) {
      doStop();
    }
  }
}
