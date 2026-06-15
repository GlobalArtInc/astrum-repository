
package org.globalart.astrum.coreui.internal.content;

import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.POST;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;

import org.globalart.astrum.coreui.AssetXO;
import org.globalart.astrum.coreui.ComponentHelper;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.query.PageResult;
import org.globalart.astrum.repository.query.QueryOptions;
import org.globalart.astrum.repository.security.RepositoryPermissionChecker;
import org.globalart.astrum.repository.security.RepositorySelector;
import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.selector.SelectorFactory;

import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.apache.shiro.authz.annotation.Logical;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.collect.Streams.stream;
import static java.util.Collections.emptyList;
import static java.util.stream.Collectors.toList;
import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import org.springframework.stereotype.Component;

@Component
@Consumes(APPLICATION_JSON)
@Produces(APPLICATION_JSON)
@Path(SelectorPreviewResource.RESOURCE_PATH)
public class SelectorPreviewResource
    implements Resource
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  static final String RESOURCE_PATH = "internal/ui/content-selectors";

  private final ComponentHelper componentHelper;

  private final RepositoryManager repositoryManager;

  private final SelectorFactory selectorFactory;

  private final RepositoryPermissionChecker repositoryPermissionChecker;

  @Autowired
  public SelectorPreviewResource(
      final ComponentHelper componentHelper,
      final RepositoryManager repositoryManager,
      final SelectorFactory selectorFactory,
      final RepositoryPermissionChecker repositoryPermissionChecker)
  {
    this.componentHelper = checkNotNull(componentHelper);
    this.repositoryManager = checkNotNull(repositoryManager);
    this.selectorFactory = checkNotNull(selectorFactory);
    this.repositoryPermissionChecker = checkNotNull(repositoryPermissionChecker);
  }

  @POST
  @Path("/preview")
  @RequiresAuthentication
  @RequiresPermissions(value = {"astrum:selectors:create", "astrum:selectors:update"}, logical = Logical.OR)
  public PageResult<SelectorPreviewAssetXO> previewContent(SelectorPreviewRequest request) {
    selectorFactory.validateSelector(request.getType().toLowerCase(), request.getExpression());

    RepositorySelector repositorySelector = RepositorySelector.fromSelector(request.getRepository());
    List<Repository> selectedRepositories = getPreviewRepositories(repositorySelector);
    if (selectedRepositories.isEmpty()) {
      return new PageResult<>(0, emptyList());
    }

    PageResult<AssetXO> result = componentHelper.previewAssets(
        repositorySelector,
        selectedRepositories,
        request.getExpression(),
        new QueryOptions(null, null, null, 0, 10));

    List<SelectorPreviewAssetXO> slimResults = result.getResults()
        .stream()
        .map(asset -> new SelectorPreviewAssetXO(asset.getName(), asset.getRepositoryName()))
        .collect(toList());
    return new PageResult<>(result.getTotal(), slimResults);
  }

  private List<Repository> getPreviewRepositories(final RepositorySelector repositorySelector) {
    List<Repository> selected;
    if (!repositorySelector.isAllRepositories()) {
      Repository specific = repositoryManager.get(repositorySelector.getName());
      selected = specific == null ? emptyList() : List.of(specific);
    }
    else if (!repositorySelector.isAllFormats()) {
      selected = stream(repositoryManager.browse())
          .filter(repository -> repository.getFormat().toString().equals(repositorySelector.getFormat()))
          .collect(toList());
    }
    else {
      selected = stream(repositoryManager.browse()).collect(toList());
    }
    return repositoryPermissionChecker.userCanBrowseRepositories(selected);
  }
}
