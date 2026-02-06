
package org.globalart.astrum.repository.content.store;

import java.lang.reflect.Constructor;
import java.lang.reflect.Parameter;
import java.lang.reflect.ParameterizedType;
import java.lang.reflect.Type;
import java.lang.reflect.TypeVariable;
import java.util.Map;
import java.util.stream.Stream;

import org.apache.commons.lang3.reflect.TypeUtils;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.BeanDefinition;
import org.springframework.beans.factory.config.ConstructorArgumentValues;
import org.springframework.beans.factory.support.AbstractBeanDefinition;
import org.springframework.beans.factory.support.BeanDefinitionBuilder;
import org.springframework.beans.factory.support.BeanDefinitionRegistry;
import org.springframework.beans.factory.support.BeanDefinitionRegistryPostProcessor;
import org.springframework.core.ResolvableType;

import static com.google.common.base.Preconditions.checkArgument;
import static org.globalart.astrum.common.text.Strings2.capitalize;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

/**
 * Support class to help create format-specific store instances in different data stores.
 */
public abstract class ContentStoreBeanDefinitionRegistryPostProcessorSupport<STORE extends ContentStoreSupport<?>>
    implements BeanDefinitionRegistryPostProcessor
{
  final Qualifier format;

  final String formatClassPrefix;

  private final Type[] formatStoreTypes;

  /**
   * Constructor used by simple format modules which only define a single store type.
   */
  protected ContentStoreBeanDefinitionRegistryPostProcessorSupport() {
    this(ContentStoreBeanDefinitionRegistryPostProcessorSupport.class);
  }

  /**
   * Use a different module class as the template defining the generic store type parameters.
   */
  @SuppressWarnings("rawtypes")
  protected ContentStoreBeanDefinitionRegistryPostProcessorSupport(
      final Class<? extends ContentStoreBeanDefinitionRegistryPostProcessorSupport> templateModule)
  {
    format = getClass().getAnnotation(Qualifier.class);

    checkArgument(format != null && !format.value().isBlank(),
        "%s must be annotated with @Qualifier(\"myformat\")", getClass());

    // extract the expected 'TitleCase' form of the format string
    formatClassPrefix = getClass().getSimpleName().substring(0, format.value().length());

    checkArgument(formatClassPrefix.equalsIgnoreCase(format.value()),
        "%s must start with %s", getClass(), capitalize(format.value()));

    formatStoreTypes = typeArguments(getClass(), templateModule)
        .filter(type -> TypeUtils.isAssignable(type, ContentStoreSupport.class))
        .toArray(Type[]::new);
  }

  @Override
  public void postProcessBeanDefinitionRegistry(final BeanDefinitionRegistry registry) {
    // bind a FormatStoreFactory for all stores declared by this module
    for (Type formatStoreType : formatStoreTypes) {
      bindFormatStoreFactory(registry, formatStoreType);
    }

    // leave FormatStoreManager binding to the main FormatStoreModule / BespokeFormatStoreModule
  }

  /**
   * Generate a {@link FormatStoreFactory} binding to produce stores of the given parameterized type.
   */
  private void bindFormatStoreFactory(final BeanDefinitionRegistry registry, final Type formatStoreType) {
    // extract the format-specific DAO class from the store type declaration: MyStore<MyDao> -> MyDao
    Class<?> formatDaoClass = (Class<?>) typeArguments(formatStoreType, ContentStoreSupport.class).findFirst()
        .orElseThrow();

    // verify the DAO class starts with the same format prefix as this module
    checkArgument(formatDaoClass.getSimpleName().startsWith(formatClassPrefix),
        "%s must start with %s", formatDaoClass, formatClassPrefix);

    // register the content store
    ResolvableType type;
    if (formatStoreType instanceof ParameterizedType pt) {
      type = ResolvableType.forClassWithGenerics((Class<?>) pt.getRawType(), formatDaoClass);
    }
    else {
      type = ResolvableType.forClass((Class<?>) formatStoreType);
    }

    AbstractBeanDefinition storeBeanDefinition = BeanDefinitionBuilder
        .rootBeanDefinition(type, null)
        .setScope(BeanDefinition.SCOPE_PROTOTYPE)
        .getBeanDefinition();
    storeBeanDefinition.setBeanClass(type.getRawClass());
    storeBeanDefinition.setConstructorArgumentValues(arguments(formatStoreType));
    registry.registerBeanDefinition(formatStoreType.getTypeName(), storeBeanDefinition);

    BeanDefinition def = BeanDefinitionBuilder.rootBeanDefinition(FormatStoreFactorySpringImpl.class)
        .addConstructorArgReference(formatStoreType.getTypeName())
        .getBeanDefinition();

    registry.registerBeanDefinition(formatDaoClass.getSimpleName(), def);
  }

  protected ConstructorArgumentValues arguments(final Type formatStoreType) {
    ConstructorArgumentValues arguments = new ConstructorArgumentValues();

    Class<?> type;
    if (formatStoreType instanceof ParameterizedType pt) {
      type = (Class<?>) pt.getRawType();
    }
    else {
      type = (Class<?>) formatStoreType;
    }

    Constructor ctor = type.getConstructors()[0];
    Parameter[] parameters = ctor.getParameters();
    for (int i = 0; i < parameters.length; i++) {
      Parameter p = parameters[i];
      Class<?> argType = p.getType();
      if (Class.class.isAssignableFrom(argType) && formatStoreType instanceof ParameterizedType pt) {
        arguments.addIndexedArgumentValue(i, pt.getActualTypeArguments()[0]);
      }
      else if (String.class.isAssignableFrom(argType)) {
        arguments.addIndexedArgumentValue(i, DEFAULT_DATASTORE_NAME);
      }
    }

    return arguments;
  }

  private static Stream<Type> typeArguments(final Type type, final Class<?> supertype) {
    Map<TypeVariable<?>, Type> typeArguments = TypeUtils.getTypeArguments(type, supertype);

    return typeArguments.values()
        .stream()
        .map(t -> TypeUtils.unrollVariables(typeArguments, t))
        .distinct();
  }
}
