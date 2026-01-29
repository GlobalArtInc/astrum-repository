
package org.globalart.astrum.repository.config;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.ConstraintValidatorContext;

import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.validation.ConstraintValidatorSupport;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * Ensure that the value matches a repository name
 *
 * @since 3.31
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class RepositoryNameValidator
    extends ConstraintValidatorSupport<RepositoryName, String>
{
  @Autowired
  private RepositoryManager repositoryManager;

  @Override
  public boolean isValid(final String value, final ConstraintValidatorContext context) {
    return repositoryManager.exists(value);
  }
}
