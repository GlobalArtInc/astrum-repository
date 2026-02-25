
package org.globalart.astrum.repository.security;

import java.util.List;

import jakarta.ws.rs.core.MediaType;
import jakarta.ws.rs.core.Response.Status;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.security.rest.ApiPrivilegeWithRepository;
import org.globalart.astrum.repository.security.rest.ApiPrivilegeWithRepositoryRequest;
import org.globalart.astrum.rest.WebApplicationMessageException;
import org.globalart.astrum.security.privilege.PrivilegeDescriptorSupport;
import org.globalart.astrum.security.privilege.rest.PrivilegeAction;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * @since 3.19
 */
public abstract class RepositoryPrivilegeDescriptorSupport<T extends ApiPrivilegeWithRepository, Y extends ApiPrivilegeWithRepositoryRequest>
    extends PrivilegeDescriptorSupport<T, Y>
{
  public static final String INVALID_REPOSITORY = "\"Invalid repository '%s' supplied.\"";

  public static final String INVALID_FORMAT_FOR_REPOSITORY = "\"Invalid format '%s' supplied for repository '%s'.\"";

  public static final String INVALID_FORMAT = "\"Invalid format '%s' supplied.\"";

  private final RepositoryManager repositoryManager;

  private final List<Format> formats;

  public RepositoryPrivilegeDescriptorSupport(
      final String type,
      final RepositoryManager repositoryManager,
      final List<Format> formats)
  {
    super(type);
    this.repositoryManager = checkNotNull(repositoryManager);
    this.formats = checkNotNull(formats);
  }

  @Override
  public void validate(final Y apiPrivilege) {
    validateActions(apiPrivilege, PrivilegeAction.getBreadActions());
    validateRepositoryAndFormat(apiPrivilege);
  }

  protected void validateRepositoryAndFormat(Y apiPrivilege) {
    String repo = apiPrivilege.getRepository();
    String format = apiPrivilege.getFormat();

    Repository repository = repositoryManager.get(repo);

    // we have an actual repo id, make sure valid
    if (!("*".equals(repo)) && repository == null) {
      throw new WebApplicationMessageException(Status.BAD_REQUEST,
          String.format(INVALID_REPOSITORY, apiPrivilege.getRepository()), MediaType.APPLICATION_JSON);
    }

    // we have an actual repo id and actual format, make sure valid
    if (!("*".equals(format)) && repository != null && !repository.getFormat().getValue().equals(format)) {
      throw new WebApplicationMessageException(Status.BAD_REQUEST,
          String.format(INVALID_FORMAT_FOR_REPOSITORY, apiPrivilege.getFormat(), apiPrivilege.getRepository()),
          MediaType.APPLICATION_JSON);
    }

    // we have an actual format and a * for repo, validate format is supported in system
    if (!("*".equals(format)) && formats.stream().noneMatch(f -> f.getValue().equals(format))) {
      throw new WebApplicationMessageException(Status.BAD_REQUEST,
          String.format(INVALID_FORMAT, apiPrivilege.getFormat()), MediaType.APPLICATION_JSON);
    }
  }
}
