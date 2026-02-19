
package org.globalart.astrum.repository.rest.api;

import java.util.Set;

import org.springframework.beans.factory.annotation.Autowired;
import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Factory for creating {@link ComponentXO} instances.
 *
 * @since 3.8
 */
@Component
public class ComponentXOFactory
{
  private final Set<ComponentXODecorator> componentXODecorators;

  @Autowired
  public ComponentXOFactory(final Set<ComponentXODecorator> componentXODecorators) {
    this.componentXODecorators = checkNotNull(componentXODecorators);
  }

  public ComponentXO createComponentXO() {
    ComponentXO componentXO = new DefaultComponentXO();
    for (ComponentXODecorator componentXODecorator : componentXODecorators) {
      componentXO = componentXODecorator.decorate(componentXO);
    }
    return componentXO;
  }
}
