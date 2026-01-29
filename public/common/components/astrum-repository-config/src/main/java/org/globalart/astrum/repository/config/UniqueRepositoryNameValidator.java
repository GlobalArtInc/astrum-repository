
package org.globalart.astrum.repository.config;

import jakarta.validation.ConstraintValidatorContext;

import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.validation.ConstraintValidatorSupport;

import static com.google.common.base.Preconditions.checkNotNull;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * Ensure that Repository names are unique case-insensitively.
 *
 * @since 3.0
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class UniqueRepositoryNameValidator
    extends ConstraintValidatorSupport<UniqueRepositoryName, String>
{
  private final RepositoryManager repositoryManager;

  @Autowired
  public UniqueRepositoryNameValidator(final RepositoryManager repositoryManager) {
    this.repositoryManager = checkNotNull(repositoryManager);
  }

  @Override
  public boolean isValid(final String value, final ConstraintValidatorContext context) {
    return !repositoryManager.exists(value);
  }
}
