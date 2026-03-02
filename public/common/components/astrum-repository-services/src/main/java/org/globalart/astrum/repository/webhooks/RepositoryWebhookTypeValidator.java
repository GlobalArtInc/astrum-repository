
package org.globalart.astrum.repository.webhooks;

import java.util.List;
import java.util.Set;
import java.util.stream.Collectors;

import jakarta.validation.ConstraintValidatorContext;

import org.globalart.astrum.validation.ConstraintValidatorSupport;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
@Component
public class RepositoryWebhookTypeValidator
    extends ConstraintValidatorSupport<RepositoryWebhookType, List<String>>
{
  private final List<RepositoryWebhook> repositoryWebhooks;

  @Autowired
  public RepositoryWebhookTypeValidator(final List<RepositoryWebhook> repositoryWebhooks) {
    this.repositoryWebhooks = repositoryWebhooks;
  }

  @Override
  public boolean isValid(final List<String> names, final ConstraintValidatorContext constraintValidatorContext) {
    if (names == null || names.isEmpty()) {
      return true; // empty list is valid
    }

    Set<String> webhookNames =
        repositoryWebhooks.stream().map(RepositoryWebhook::getName).collect(Collectors.toUnmodifiableSet());
    // Check if all names in the provided list are valid webhook names
    return webhookNames.containsAll(names);
  }
}
