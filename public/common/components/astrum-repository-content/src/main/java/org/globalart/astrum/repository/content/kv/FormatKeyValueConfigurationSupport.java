
package org.globalart.astrum.repository.content.kv;

import org.globalart.astrum.repository.content.store.ContentStoreBeanDefinitionRegistryPostProcessorSupport;

/**
 * A support class formats can implement and use {@link Configuration} annotation to create the key-value table for the
 * format.
 */
public abstract class FormatKeyValueConfigurationSupport<KEY_VALUE_STORE extends KeyValueStore<?>>
    extends ContentStoreBeanDefinitionRegistryPostProcessorSupport<KEY_VALUE_STORE>
{
  protected FormatKeyValueConfigurationSupport() {
    super(FormatKeyValueConfigurationSupport.class);
  }
}
