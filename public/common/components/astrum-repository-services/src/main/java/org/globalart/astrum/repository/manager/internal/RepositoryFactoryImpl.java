
package org.globalart.astrum.repository.manager.internal;

import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;

import org.springframework.stereotype.Component;

@Component
public class RepositoryFactoryImpl
    implements RepositoryFactory
{
  private final EventManager eventManager;

  public RepositoryFactoryImpl(final EventManager eventManager) {
    this.eventManager = eventManager;
  }

  @Override
  public Repository create(final Type type, final Format format) {
    return new RepositoryImpl(eventManager, type, format);
  }
}
