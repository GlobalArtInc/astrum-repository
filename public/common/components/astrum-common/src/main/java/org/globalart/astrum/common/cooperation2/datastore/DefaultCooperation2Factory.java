
package org.globalart.astrum.common.cooperation2.datastore;

import java.util.Arrays;
import java.util.stream.Collectors;

import org.globalart.astrum.common.cooperation2.Cooperation2;
import org.globalart.astrum.common.cooperation2.Cooperation2Factory;
import org.globalart.astrum.common.cooperation2.datastore.internal.LocalCooperation2;
import org.globalart.astrum.common.cooperation2.internal.DisabledCooperation2;
import org.globalart.astrum.common.cooperation2.internal.MutableConfigSupport;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * @since 3.41
 */
@Component
@Qualifier("local")
public class DefaultCooperation2Factory
    implements Cooperation2Factory
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Override
  public Builder configure() {
    return new DefaultCooperation2Builder();
  }

  protected class DefaultCooperation2Builder
      extends MutableConfigSupport
  {
    @Override
    public Cooperation2 build(final String id) {
      if (!enabled) {
        log.debug("Disabled cooperation: {}", id);
        return new DisabledCooperation2(id);
      }
      return new LocalCooperation2(id, this.copy());
    }

    @Override
    public Cooperation2 build(final Class<?> id, final String... keys) {
      if (!enabled) {
        log.debug("Disabled cooperation: {}", id);
        return new DisabledCooperation2(stripGuice(id, keys));
      }
      return new LocalCooperation2(stripGuice(id, keys), this.copy());
    }
  }

  /*
   * When classes are enhanced by Guice AOP they can have random strings and we need them to be consistent
   */
  protected static String stripGuice(final Class<?> clazz, final String... keys) {
    String simpleName = clazz.getSimpleName();
    // this is the normal case due to method interceptors
    if (simpleName.contains("EnhancerByGuice") && clazz.getSuperclass() != null) {
      return stripGuice(clazz.getSuperclass(), keys);
    }

    return Arrays.asList(keys).stream().collect(Collectors.joining("-", simpleName + '-', ""));
  }
}
