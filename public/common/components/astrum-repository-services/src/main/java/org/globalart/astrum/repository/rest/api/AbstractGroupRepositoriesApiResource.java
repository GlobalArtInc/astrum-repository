
package org.globalart.astrum.repository.rest.api;

import java.util.Collection;
import java.util.Set;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.ConstraintViolation;
import jakarta.ws.rs.BeanParam;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.POST;
import jakarta.ws.rs.PUT;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.PathParam;
import jakarta.ws.rs.core.Response;

import org.globalart.astrum.common.app.ApplicationVersion;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.rest.GroupRepositoryApiRequestToConfigurationConverter;
import org.globalart.astrum.repository.rest.api.model.AbstractApiRepository;
import org.globalart.astrum.repository.rest.api.model.GroupAttributes;
import org.globalart.astrum.repository.rest.api.model.GroupDeployAttributes;
import org.globalart.astrum.repository.rest.api.model.GroupRepositoryApiRequest;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.validation.ConstraintViolationFactory;

import com.google.common.collect.Sets;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.Parameter;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Objects.nonNull;
import static org.globalart.astrum.validation.ConstraintViolations.maybePropagate;

/**
 * @since 3.20
 */
public abstract class AbstractGroupRepositoriesApiResource<T extends GroupRepositoryApiRequest>
    extends AbstractRepositoriesApiResource<T>
{
  private static final String WRITABLE_MEMBER = "writableMember";

  private ConstraintViolationFactory constraintViolationFactory;

  private ApplicationVersion applicationVersion;

  protected GroupRepositoryApiRequestToConfigurationConverter<T> configurationConverter;

  @Autowired
  public void setConstraintViolationFactory(final ConstraintViolationFactory constraintViolationFactory) {
    this.constraintViolationFactory = checkNotNull(constraintViolationFactory);
  }

  @Autowired
  public void setApplicationVersion(final ApplicationVersion applicationVersion) {
    this.applicationVersion = applicationVersion;
  }

  @Autowired
  public void setConfigurationConverter(
      final GroupRepositoryApiRequestToConfigurationConverter<T> configurationConverter)
  {
    this.configurationConverter = configurationConverter;
  }

  @Override
  public Configuration adapt(final T request) {
    return configurationConverter.convert(request);
  }

  @POST
  @Override
  public Response createRepository(final T request) {
    validateRequest(request);
    return super.createRepository(request);
  }

  @PUT
  @Path("/{repositoryName}")
  @Override
  public Response updateRepository(
      final T request,
      @PathParam("repositoryName") final String repositoryName)
  {
    validateRequest(request);
    return super.updateRepository(request, repositoryName);
  }

  private void validateRequest(T request) {
    String groupFormat = request.getFormat();
    Set<ConstraintViolation<?>> violations = Sets.newHashSet();
    Collection<String> memberNames = request.getGroup().getMemberNames();
    for (String repositoryName : memberNames) {
      Repository repository = repositoryManager.get(repositoryName);
      if (nonNull(repository)) {
        String memberFormat = repository.getFormat().getValue();
        if (!memberFormat.equals(groupFormat)) {
          violations.add(constraintViolationFactory.createViolation("memberNames",
              "Member repository format does not match group repository format: " + repositoryName));
        }
      }
      else {
        violations.add(constraintViolationFactory.createViolation("memberNames",
            "Member repository does not exist: " + repositoryName));
      }
    }
    validateGroupDeploy(request, violations);
    maybePropagate(violations, log);
  }

  private void validateGroupDeploy(T request, Set<ConstraintViolation<?>> violations) {
    GroupAttributes group = request.getGroup();
    if (group instanceof GroupDeployAttributes) {
      GroupDeployAttributes groupDeployAttributes = (GroupDeployAttributes) group;
      final String writableMember = groupDeployAttributes.getWritableMember();
      if (writableMember == null || writableMember.isEmpty()) {
        return;
      }
      Repository writableMemberRepository = repositoryManager.get(writableMember);
      if (writableMemberRepository == null) {
        violations.add(constraintViolationFactory.createViolation(
            WRITABLE_MEMBER,
            "Writable member repository does not exist"));
      }
      else if (!writableMemberRepository.getType().getValue().equals(HostedType.NAME)) {
        violations.add(constraintViolationFactory.createViolation(
            WRITABLE_MEMBER,
            "Writable member must be a hosted repository"));
      }
      else if (!writableMemberRepository.getFormat().getValue().equals(request.getFormat())) {
        violations.add(constraintViolationFactory.createViolation(
            WRITABLE_MEMBER,
            "Writable member repository format does not match group repository format: " + writableMember));
      }
      else if (!request.getGroup().getMemberNames().contains(writableMember)) {
        violations.add(constraintViolationFactory.createViolation(
            WRITABLE_MEMBER,
            "Writable member must be a member of the group"));
      }
    }
  }

  @GET
  @Path("/{repositoryName}")
  @Operation(summary = "Get repository")
  @Override
  public AbstractApiRepository getRepository(
      @Parameter(hidden = true) @BeanParam final FormatAndType formatAndType,
      @PathParam("repositoryName") final String repositoryName)
  {
    return super.getRepository(formatAndType, repositoryName);
  }
}
