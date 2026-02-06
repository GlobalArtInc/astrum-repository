
package org.globalart.astrum.repository.content.search;

import java.util.Optional;
import java.util.stream.Stream;

import org.globalart.astrum.common.entity.EntityHelper;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.fluent.FluentComponents;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Strings.nullToEmpty;
import static java.util.Optional.empty;

/**
 * @since 3.26
 */
@Component
@Qualifier("default")
public class DefaultComponentFinder
    implements ComponentFinder
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Override
  public Stream<FluentComponent> findComponentsByModel(
      final Repository repository,
      final String searchComponentId,
      final String namespace,
      final String name,
      final String version)
  {
    Optional<FluentComponent> component = empty();

    if (repository != null) {
      FluentComponents components = contentFacet(repository).components();

      if (searchComponentId != null) {
        component = components.find(EntityHelper.id(searchComponentId));
      }

      if (!component.isPresent()) {
        component = components
            .name(name)
            .namespace(nullToEmpty(namespace))
            .version(nullToEmpty(version))
            .find();
      }
    }

    return component.map(Stream::of).orElse(Stream.empty());
  }

  protected ContentFacet contentFacet(final Repository repository) {
    return repository.facet(ContentFacet.class);
  }
}
