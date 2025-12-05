
package org.globalart.astrum.internal.capability;

import java.util.Collection;
import java.util.Set;
import java.util.concurrent.CopyOnWriteArraySet;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.capability.CapabilityDescriptor;
import org.globalart.astrum.capability.CapabilityDescriptorRegistry;
import org.globalart.astrum.capability.CapabilityType;
import org.globalart.astrum.capability.DefaultCapabilityDescriptorProvider;

import com.google.common.collect.Lists;
import org.springframework.context.annotation.Primary;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Primary
@Component
class DefaultCapabilityDescriptorRegistry
    implements CapabilityDescriptorRegistry
{
  private final DefaultCapabilityDescriptorProvider defaultCapabilityDescriptorProvider;

  private final Set<CapabilityDescriptor> dynamicDescriptors;

  @Autowired
  DefaultCapabilityDescriptorRegistry(final DefaultCapabilityDescriptorProvider defaultCapabilityDescriptorProvider) {
    this.defaultCapabilityDescriptorProvider = checkNotNull(defaultCapabilityDescriptorProvider);
    this.dynamicDescriptors = new CopyOnWriteArraySet<CapabilityDescriptor>();
  }

  @Override
  public CapabilityDescriptorRegistry register(final CapabilityDescriptor capabilityDescriptor) {
    dynamicDescriptors.add(checkNotNull(capabilityDescriptor));
    return this;
  }

  @Override
  public CapabilityDescriptorRegistry unregister(final CapabilityDescriptor capabilityDescriptor) {
    dynamicDescriptors.remove(checkNotNull(capabilityDescriptor));
    return this;
  }

  @Override
  public CapabilityDescriptor get(final CapabilityType capabilityType) {
    final CapabilityDescriptor descriptor = get(defaultCapabilityDescriptorProvider.get(), capabilityType);
    if (descriptor == null) {
      return get(dynamicDescriptors, capabilityType);
    }
    return descriptor;
  }

  @Override
  public CapabilityDescriptor[] getAll() {
    final Collection<CapabilityDescriptor> all = Lists.newArrayList();
    all.addAll(defaultCapabilityDescriptorProvider.get());
    all.addAll(dynamicDescriptors);

    return all.toArray(new CapabilityDescriptor[all.size()]);
  }

  private CapabilityDescriptor get(
      final Collection<CapabilityDescriptor> descriptors,
      final CapabilityType capabilityType)
  {
    for (final CapabilityDescriptor descriptor : descriptors) {
      if (descriptor.type().equals(capabilityType)) {
        return descriptor;
      }
    }
    return null;
  }
}
