
package org.globalart.astrum.coreui;

import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.stream.Collectors;
import java.util.stream.StreamSupport;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.encoding.EncodingUtil;
import org.globalart.astrum.extdirect.DirectComponent;
import org.globalart.astrum.extdirect.DirectComponentSupport;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.browse.node.BrowseNode;
import org.globalart.astrum.repository.browse.node.BrowseNodeConfiguration;
import org.globalart.astrum.repository.browse.node.BrowseNodeQueryService;
import org.globalart.astrum.repository.manager.RepositoryManager;

import com.codahale.metrics.annotation.ExceptionMetered;
import com.codahale.metrics.annotation.Timed;
import com.softwarementors.extjs.djn.config.annotations.DirectAction;
import com.softwarementors.extjs.djn.config.annotations.DirectMethod;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Browse {@link DirectComponent}.
 *
 * @since 3.6
 */
@Component
@DirectAction(action = "coreui_Browse")
public class BrowseComponent
    extends DirectComponentSupport
{
  static final String FOLDER = "folder";

  static final String COMPONENT = "component";

  static final String ASSET = "asset";

  private final BrowseNodeConfiguration configuration;

  private final BrowseNodeQueryService browseNodeQueryService;

  private final RepositoryManager repositoryManager;

  @Autowired
  public BrowseComponent(
      final BrowseNodeConfiguration browseNodeConfiguration,
      final BrowseNodeQueryService browseNodeQueryService,
      final RepositoryManager repositoryManager)
  {
    this.configuration = checkNotNull(browseNodeConfiguration);
    this.browseNodeQueryService = checkNotNull(browseNodeQueryService);
    this.repositoryManager = checkNotNull(repositoryManager);
  }

  @DirectMethod
  @Timed
  @ExceptionMetered
  public List<BrowseNodeXO> read(TreeStoreLoadParameters treeStoreLoadParameters) {
    String repositoryName = treeStoreLoadParameters.getRepositoryName();
    final String path = treeStoreLoadParameters.getNode();

    Repository repository = repositoryManager.get(repositoryName);
    if (repository == null) {
      log.debug("Repository '{}' not found, returning empty browse result", repositoryName);
      return Collections.emptyList();
    }

    List<String> pathSegments;
    if (isRoot(path)) {
      pathSegments = Collections.emptyList();
    }
    else {
      pathSegments = Arrays.stream(path.split("/"))
          .map(EncodingUtil::urlDecode)
          .collect(Collectors.toList()); // NOSONAR
    }

    return StreamSupport.stream(
        browseNodeQueryService.getByPath(repository, pathSegments, configuration.getMaxNodes()).spliterator(), false)
        .map(browseNode -> {
          String encodedPath = EncodingUtil.urlEncode(browseNode.getName());
          String type = getNodeType(browseNode);
          return new BrowseNodeXO()
              .withId(isRoot(path) ? encodedPath : (path + "/" + encodedPath))
              .withType(type)
              .withText(browseNode.getName())
              .withLeaf(browseNode.isLeaf())
              .withComponentId(browseNode.getComponentId() == null ? null : browseNode.getComponentId().getValue())
              .withAssetId(browseNode.getAssetId() == null ? null : browseNode.getAssetId().getValue())
              .withPackageUrl(browseNode.getPackageUrl());
        })
        .collect(Collectors.toList()); // NOSONAR
  }

  public boolean isRoot(String path) {
    return "/".equals(path);
  }

  private String getNodeType(BrowseNode browseNode) {
    if (browseNode.getAssetId() != null) {
      return ASSET;
    }
    else if (browseNode.getComponentId() != null) {
      return COMPONENT;
    }
    else {
      return FOLDER;
    }
  }
}
