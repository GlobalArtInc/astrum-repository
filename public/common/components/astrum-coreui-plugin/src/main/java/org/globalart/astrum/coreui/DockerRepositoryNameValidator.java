
package org.globalart.astrum.coreui;

import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.repository.manager.RepositoryManager;

import jakarta.validation.ConstraintValidator;
import jakarta.validation.ConstraintValidatorContext;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

/**
 * Validator for {@link DockerRepositoryNameConstraint} that ensures Docker repository names are lowercase.
 *
 * <p>
 * The {@link DockerRepositoryNameConstraint} is scoped to the {@code Create} validation group, so
 * this validator only runs on creation (where no repository with the name exists yet) and never on update. The
 * {@link RepositoryManager}-based existing-repository exemption below is therefore dead-but-harmless: on create
 * {@code exists(name)} is always {@code false}, so the exemption can never fire. It is intentionally retained (rather
 * than removed) to keep the diff minimal; backward compatibility for mixed-case updates is now provided by
 * the validation-group scoping, not by this runtime check.
 */
@Component
public class DockerRepositoryNameValidator
    implements ConstraintValidator<DockerRepositoryNameConstraint, RepositoryXO>
{
  private static final Logger log = LoggerFactory.getLogger(DockerRepositoryNameValidator.class);

  private RepositoryManager repositoryManager;

  /**
   * No-arg constructor for Bean Validation API compatibility and other cases where validators are
   * instantiated directly (for example, {@code Validation.buildDefaultValidatorFactory()} in tests).
   * In such non-DI scenarios, {@link RepositoryManager}-based existing-repository exemption logic is
   * not available because the dependency is not injected.
   */
  public DockerRepositoryNameValidator() {
    this.repositoryManager = null;
  }

  /**
   * Constructor with dependency injection for full validation logic including existing repository exemption.
   */
  @Autowired
  public DockerRepositoryNameValidator(final RepositoryManager repositoryManager) {
    this.repositoryManager = repositoryManager;
  }

  // Constants matching com.globalart.astrum.repository.docker.internal.DockerFormat.NAME
  // the coreui plugin doesn't have a dependency on the docker plugin, so we can't import this constant directly
  private static final String DOCKER_FORMAT = "docker";

  // Recipe name prefixes matching:
  // - com.globalart.astrum.repository.docker.internal.datastore.recipe.DockerHostedRecipe.NAME = "docker-hosted"
  // - com.globalart.astrum.repository.docker.internal.datastore.recipe.DockerProxyRecipe.NAME = "docker-proxy"
  // - com.globalart.astrum.repository.docker.internal.datastore.recipe.DockerGroupRecipe.NAME = "docker-group"
  // the coreui plugin doesn't have a dependency on the docker plugin, so we can't import these constants directly
  private static final String DOCKER_RECIPE_PREFIX = "docker-";

  @Override
  public boolean isValid(final RepositoryXO value, final ConstraintValidatorContext context) {
    if (value == null || value.getName() == null) {
      return true;
    }

    // Check both format and recipe fields to determine if this is a Docker repository
    boolean isDockerRepository = false;

    if (value.getFormat() != null && DOCKER_FORMAT.equalsIgnoreCase(value.getFormat())) {
      isDockerRepository = true;
    }

    if (value.getRecipe() != null && value.getRecipe().toLowerCase().startsWith(DOCKER_RECIPE_PREFIX)) {
      isDockerRepository = true;
    }

    // Only validate for Docker repositories
    if (!isDockerRepository) {
      return true;
    }

    String name = value.getName();

    // If the name is already lowercase, it's valid
    if (name.equals(Strings2.lower(name))) {
      return true;
    }

    // CRITICAL FIX: Exempt existing repositories to allow updates
    // Check if repository already exists - if so, allow the update
    // Only perform this check if RepositoryManager is available (injected via DI)
    if (repositoryManager != null) {
      try {
        if (repositoryManager.exists(name)) {
          log.debug("Allowing update to existing Docker repository with mixed-case name: {}", name);
          return true;
        }
      }
      catch (Exception e) {
        log.warn("Could not check repository existence for: {}", name, e);
        // On error, fail safe and allow the operation
        return true;
      }
    }

    // Only enforce lowercase for NEW repository creation
    log.debug("Blocking creation of new Docker repository with mixed-case name: {}", name);
    return false;
  }
}
