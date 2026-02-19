
package org.globalart.astrum.repository.rest.api;

import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.StringJoiner;
import java.util.function.Function;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.ConstraintViolationException;
import jakarta.validation.Valid;
import jakarta.validation.constraints.NotNull;
import jakarta.validation.groups.Default;
import jakarta.ws.rs.BeanParam;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.POST;
import jakarta.ws.rs.PUT;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.PathParam;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.core.Response;
import jakarta.ws.rs.core.Response.Status;

import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.repository.HighAvailabilitySupportChecker;
import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.repository.Recipe;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.rest.api.model.AbstractApiRepository;
import org.globalart.astrum.repository.rest.api.model.AbstractRepositoryApiRequest;
import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.rest.ValidationErrorsException;
import org.globalart.astrum.rest.WebApplicationMessageException;
import org.globalart.astrum.validation.Validate;
import org.globalart.astrum.validation.group.Create;

import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.Parameter;
import org.apache.shiro.authc.AuthenticationException;
import org.apache.shiro.authz.AuthorizationException;
import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.springframework.beans.factory.annotation.Value;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import static jakarta.ws.rs.core.Response.Status.BAD_REQUEST;
import static jakarta.ws.rs.core.Response.Status.METHOD_NOT_ALLOWED;
import static jakarta.ws.rs.core.Response.Status.NOT_FOUND;
import static org.globalart.astrum.rest.ApiDocConstants.BLOBSTORE_CHANGE_NOT_ALLOWED;
import static org.globalart.astrum.rest.ApiDocConstants.BLOBSTORE_NOT_FOUND;

/**
 * @since 3.20
 */
@Produces(APPLICATION_JSON)
@Consumes(APPLICATION_JSON)
public abstract class AbstractRepositoriesApiResource<T extends AbstractRepositoryApiRequest>
    implements Resource
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private AuthorizingRepositoryManager authorizingRepositoryManager;

  protected RepositoryManager repositoryManager;

  protected BlobStoreManager blobStoreManager;

  private Map<String, ApiRepositoryAdapter> convertersByFormat;

  private ApiRepositoryAdapter defaultAdapter;

  protected HighAvailabilitySupportChecker highAvailabilitySupportChecker;

  private Map<String, Recipe> recipesByFormat;

  private boolean blobStoreValidation = true;

  @Autowired
  public void setHighAvailabilitySupportChecker(final HighAvailabilitySupportChecker highAvailabilitySupportChecker) {
    this.highAvailabilitySupportChecker = highAvailabilitySupportChecker;
  }

  @Autowired
  public void setAuthorizingRepositoryManager(final AuthorizingRepositoryManager authorizingRepositoryManager) {
    this.authorizingRepositoryManager = checkNotNull(authorizingRepositoryManager);
  }

  @Autowired
  public void setRepositoryManager(final RepositoryManager repositoryManager) {
    this.repositoryManager = checkNotNull(repositoryManager);
  }

  @Autowired
  public void setBlobStoreManager(final BlobStoreManager blobStoreManager) {
    this.blobStoreManager = blobStoreManager;
  }

  @Autowired
  public void setConvertersByFormat(final List<ApiRepositoryAdapter> convertersByFormatList) {
    this.convertersByFormat = QualifierUtil.buildQualifierBeanMap(checkNotNull(convertersByFormatList));
  }

  @Autowired
  public void setDefaultAdapter(final ApiRepositoryAdapter defaultAdapter) {
    this.defaultAdapter = checkNotNull(defaultAdapter);
  }

  @Autowired
  public void setRecipesByFormat(final List<Recipe> recipesByFormatList) {
    this.recipesByFormat = QualifierUtil.buildQualifierBeanMap(checkNotNull(recipesByFormatList));
  }

  @Autowired
  public void setBlobStoreValidation(
      @Value("${astrum.api.validateBlobStore:true}") final boolean blobStoreValidation)
  {
    this.blobStoreValidation = blobStoreValidation;
  }

  /**
   * Adapts the request to a configuration object.
   *
   * @param request
   * @return the request as a configuration object.
   */
  public abstract Configuration adapt(final T request);

  @POST
  @RequiresAuthentication
  @Validate(groups = {Create.class, Default.class})
  public Response createRepository(@NotNull @Valid final T request) {
    verifyAPIEnabled(request.getFormat());
    try {
      Configuration configuration = adapt(request);
      validateRequest(configuration, configuration.getRepositoryName());
      authorizingRepositoryManager.create(configuration);
      return Response.status(Status.CREATED).build();
    }
    catch (AuthorizationException | AuthenticationException | ConstraintViolationException e) {
      throw e;
    }
    catch (Exception e) {
      StringJoiner stringJoiner = new StringJoiner("\n", "\"", "\"");
      stringJoiner.add(e.getMessage());
      for (Throwable t : e.getSuppressed()) {
        stringJoiner.add(t.getMessage());
      }
      String message = stringJoiner.toString();
      log.debug("Failed to create a new repository via REST: {}", message, e);
      throw new WebApplicationMessageException(BAD_REQUEST, message, APPLICATION_JSON);
    }
  }

  @PUT
  @Path("/{repositoryName}")
  @RequiresAuthentication
  @Validate
  public Response updateRepository(
      @NotNull @Valid final T request,
      @PathParam("repositoryName") final String repositoryName)
  {
    try {
      Configuration newConfiguration = adapt(request);
      validateRequest(newConfiguration, repositoryName);
      boolean updated = authorizingRepositoryManager.update(newConfiguration);

      Status status = updated ? Status.NO_CONTENT : Status.NOT_FOUND;
      return Response.status(status).build();
    }
    catch (AuthorizationException | AuthenticationException | ConstraintViolationException e) {
      throw e;
    }
    catch (Exception e) {
      StringJoiner stringJoiner = new StringJoiner("\n", "\"", "\"");
      stringJoiner.add(e.getMessage());
      for (Throwable t : e.getSuppressed()) {
        stringJoiner.add(t.getMessage());
      }
      String message = stringJoiner.toString();
      log.debug("Failed to edit a repository via REST: {}", message, e);

      throw new WebApplicationMessageException(BAD_REQUEST, message, APPLICATION_JSON);
    }
  }

  @GET
  @Path("/{repositoryName}")
  @RequiresAuthentication
  @Validate
  @Operation(summary = "Get repository")
  public AbstractApiRepository getRepository(
      @Parameter(hidden = true) @BeanParam final FormatAndType formatAndType,
      @PathParam("repositoryName") final String repositoryName)
  {
    return authorizingRepositoryManager.getRepositoryWithAdmin(repositoryName)
        .filter(r -> r.getType().getValue().equals(formatAndType.type()) &&
            r.getFormat().getValue().equals(formatAndType.format()))
        .map(r -> convertersByFormat.getOrDefault(r.getFormat().toString(), defaultAdapter).adaptDecorated(r))
        .orElseThrow(() -> new WebApplicationMessageException(NOT_FOUND, "\"Repository not found\"", APPLICATION_JSON));
  }

  /**
   * By default, the API is enabled in High Availability, otherwise it should be overridden by a format.
   * 
   * @return {@code true} in case of API is enabled or {@code false} otherwise.
   */
  public boolean isApiEnabled() {
    return true;
  }

  private void validateRequest(final Configuration newConfig, final String repositoryName) {
    validateFormatEnabled(newConfig.getRecipeName());
    ensureRepositoryNameMatches(newConfig, repositoryName);
    if (blobStoreValidation) {
      validateBlobStoreName(newConfig, repositoryName);
    }
  }

  private void ensureRepositoryNameMatches(final Configuration newConfig, final String repositoryName) {
    if (!repositoryName.equals(newConfig.getRepositoryName())) {
      throw new ValidationErrorsException("name", "Renaming a repository is not supported");
    }
  }

  private void validateBlobStoreName(final Configuration newConfiguration, final String repositoryName) {
    String blobStoreName = (String) newConfiguration.getAttributes().get("storage").get("blobStoreName");
    if (!blobStoreManager.exists(blobStoreName)) {
      throw new ValidationErrorsException("BlobStoreName", BLOBSTORE_NOT_FOUND);
    }

    Repository repository = repositoryManager.get(repositoryName);
    if (repository == null) {
      return;
    }

    Function<Map<String, Map<String, Object>>, String> getBlobStorageFromAttributes =
        storageAttr -> (String) storageAttr.get("storage").get("blobStoreName");

    String newBlobStoreName = Optional.ofNullable(newConfiguration.getAttributes())
        .map(getBlobStorageFromAttributes)
        .orElse("");
    String currentBlobStore = getBlobStorageFromAttributes.apply(repository.getConfiguration().getAttributes());

    if (!newBlobStoreName.equals(currentBlobStore)) {
      throw new ValidationErrorsException("BlobStoreName", BLOBSTORE_CHANGE_NOT_ALLOWED);
    }
  }

  private void verifyAPIEnabled(final String format) {
    if (!isApiEnabled()) {
      String message = String.format("Format %s is disabled in High Availability", format);
      throw new WebApplicationMessageException(METHOD_NOT_ALLOWED, message, APPLICATION_JSON);
    }
  }

  private void validateFormatEnabled(final String recipeName) {
    Recipe recipe = recipesByFormat.get(recipeName);
    if (recipe != null && !recipe.isFeatureEnabled()) {
      throw new ValidationErrorsException("This format is not currently enabled");
    }
  }

}
