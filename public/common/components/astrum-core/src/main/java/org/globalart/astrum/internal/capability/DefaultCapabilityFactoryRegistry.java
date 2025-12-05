
package org.globalart.astrum.internal.capability;

import java.util.List;
import java.util.Map;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.capability.Capability;
import org.globalart.astrum.capability.CapabilityDescriptor;
import org.globalart.astrum.capability.CapabilityDescriptorRegistry;
import org.globalart.astrum.capability.CapabilityFactory;
import org.globalart.astrum.capability.CapabilityFactoryRegistry;
import org.globalart.astrum.capability.CapabilityType;
import org.globalart.astrum.common.QualifierUtil;

import com.google.common.collect.Maps;
import org.springframework.beans.BeansException;
import org.springframework.context.ApplicationContext;
import org.springframework.context.ApplicationContextAware;
import org.springframework.context.annotation.Primary;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Default {@link CapabilityFactoryRegistry} implementation.
 *
 * @since capabilities 2.0
 */
@Primary
@Component
class DefaultCapabilityFactoryRegistry
    implements CapabilityFactoryRegistry, ApplicationContextAware
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final Map<String, CapabilityFactory> factories;

  private final Map<String, CapabilityFactory> dynamicFactories;

  private final CapabilityDescriptorRegistry capabilityDescriptorRegistry;

  private ApplicationContext context;

  private Map<String, String> capabilityByQualifier;

  @Autowired
  DefaultCapabilityFactoryRegistry(
      final List<CapabilityFactory> factoriesList,
      final CapabilityDescriptorRegistry capabilityDescriptorRegistry)
  {
    this.capabilityDescriptorRegistry = checkNotNull(capabilityDescriptorRegistry);
    this.factories = QualifierUtil.buildQualifierBeanMap(checkNotNull(factoriesList));
    this.dynamicFactories = Maps.newConcurrentMap();
  }

  @Override
  public CapabilityFactoryRegistry register(final CapabilityType type, final CapabilityFactory factory) {
    checkNotNull(factory);
    checkArgument(!factories.containsKey(type.toString()), "Factory already registered for %s", type);
    checkArgument(!dynamicFactories.containsKey(type.toString()), "Factory already registered for %s", type);

    dynamicFactories.put(type.toString(), factory);
    log.debug("Added {} -> {}", type, factory);

    return this;
  }

  @Override
  public CapabilityFactoryRegistry unregister(final CapabilityType type) {
    if (type != null) {
      final CapabilityFactory factory = dynamicFactories.remove(type.toString());
      log.debug("Removed {} -> {}", type, factory);
    }

    return this;
  }

  @Override
  public CapabilityFactory get(final CapabilityType type) {
    CapabilityFactory factory = factories.get(checkNotNull(type).toString());
    if (factory == null) {
      factory = dynamicFactories.get(checkNotNull(type).toString());
    }
    if (factory == null) {
      final CapabilityDescriptor descriptor = capabilityDescriptorRegistry.get(type);
      if (descriptor instanceof CapabilityFactory) {
        factory = (CapabilityFactory) descriptor;
      }
      if (factory == null) {
        factory = () -> context.getBean(type.toString(), Capability.class);
      }
    }
    return factory;
  }

  @Override
  public void setApplicationContext(final ApplicationContext applicationContext) throws BeansException {
    this.context = applicationContext;
  }
}
