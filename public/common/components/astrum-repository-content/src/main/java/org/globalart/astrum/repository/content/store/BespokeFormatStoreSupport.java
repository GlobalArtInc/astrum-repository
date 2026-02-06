
package org.globalart.astrum.repository.content.store;

import org.globalart.astrum.repository.content.browse.store.BrowseNodeStore;

import net.bytebuddy.ByteBuddy;
import org.springframework.beans.factory.support.AbstractBeanDefinition;
import org.springframework.beans.factory.support.AutowireCandidateQualifier;
import org.springframework.beans.factory.support.BeanDefinitionBuilder;
import org.springframework.beans.factory.support.BeanDefinitionRegistry;

@SuppressWarnings({"rawtypes", "unchecked", "unused"})
public abstract class BespokeFormatStoreSupport<CONTENT_REPOSITORY_STORE extends ContentRepositoryStore<?>, COMPONENT_STORE extends ComponentStore<?>, ASSET_STORE extends AssetStore<?>, ASSET_BLOB_STORE extends AssetBlobStore<?>, BROWSE_STORE extends BrowseNodeStore<?>>
    extends ContentStoreBeanDefinitionRegistryPostProcessorSupport
{
  protected BespokeFormatStoreSupport() {
    super(BespokeFormatStoreSupport.class);
  }

  @Override
  public void postProcessBeanDefinitionRegistry(final BeanDefinitionRegistry registry) {
    super.postProcessBeanDefinitionRegistry(registry);

    AbstractBeanDefinition def = BeanDefinitionBuilder.rootBeanDefinition(createType())
        .addConstructorArgValue(formatClassPrefix)
        .setAutowireMode(AbstractBeanDefinition.AUTOWIRE_BY_TYPE)
        .getBeanDefinition();

    def.addQualifier(new AutowireCandidateQualifier(getClass(), def));

    registry.registerBeanDefinition(format + FormatStoreManager.class.getSimpleName(), def);
  }

  private Class<? extends FormatStoreManager> createType() {
    return new ByteBuddy().subclass(FormatStoreManager.class)
        .annotateType(format)
        .make()
        .load(getClass().getClassLoader())
        .getLoaded();
  }
}
