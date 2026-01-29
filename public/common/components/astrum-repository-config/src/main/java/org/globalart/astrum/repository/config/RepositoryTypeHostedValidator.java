
package org.globalart.astrum.repository.config;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.ConstraintValidatorContext;

import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.types.HostedType;
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
public class RepositoryTypeHostedValidator
    extends ConstraintValidatorSupport<RepositoryTypeHosted, String>
{
  @Autowired
  private RepositoryManager repositoryManager;

  @Override
  public boolean isValid(final String value, final ConstraintValidatorContext context) {
    if (!Strings2.isBlank(value)) {
      Repository repository = repositoryManager.get(value);

      if (repository != null) {
        return repository.getType().getValue().equals(HostedType.NAME);
      }
      return false;
    }
    else {
      // let the notblank validator do its job
      return true;
    }
  }
}
