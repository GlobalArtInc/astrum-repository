
package org.globalart.astrum.repository.capability.internal;

import java.util.function.Supplier;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.capability.Condition;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.repository.capability.RepositoryConditions;
import org.globalart.astrum.repository.manager.RepositoryManager;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Default implementation of {@link RepositoryConditions}.
 *
 * @since 3.0
 */
@Component
public class RepositoryConditionsImpl
    implements RepositoryConditions
{
  private final EventManager eventManager;

  private final RepositoryManager repositoryManager;

  @Autowired
  public RepositoryConditionsImpl(
      final EventManager eventManager,
      final RepositoryManager repositoryManager)
  {
    this.eventManager = checkNotNull(eventManager);
    this.repositoryManager = checkNotNull(repositoryManager);
  }

  @Override
  public Condition repositoryIsOnline(final Supplier<String> repositoryName) {
    return new RepositoryOnlineCondition(eventManager, repositoryManager, repositoryName);
  }

  @Override
  public Condition repositoryExists(final Supplier<String> repositoryName) {
    return new RepositoryExistsCondition(eventManager, repositoryManager, repositoryName);
  }
}
