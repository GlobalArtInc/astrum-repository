
package org.globalart.astrum.repository.rest.internal.resources;

import java.net.URL;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.PathParam;
import jakarta.ws.rs.Produces;
import jakarta.ws.rs.WebApplicationException;
import jakarta.ws.rs.core.Context;
import jakarta.ws.rs.core.Response;
import jakarta.ws.rs.core.UriBuilder;
import jakarta.ws.rs.core.UriInfo;

import org.globalart.astrum.common.template.TemplateHelper;
import org.globalart.astrum.common.template.TemplateParameters;
import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.browse.node.BrowseListItem;
import org.globalart.astrum.repository.browse.node.BrowseNode;
import org.globalart.astrum.repository.browse.node.BrowseNodeConfiguration;
import org.globalart.astrum.repository.browse.node.BrowseNodeQueryService;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.security.RepositoryViewPermission;
import org.globalart.astrum.rest.Resource;
import org.globalart.astrum.security.SecurityHelper;

import com.google.common.collect.Iterables;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Arrays.asList;
import static jakarta.ws.rs.core.MediaType.TEXT_HTML;
import static jakarta.ws.rs.core.Response.Status.NOT_FOUND;
import static org.globalart.astrum.common.encoding.EncodingUtil.urlEncode;
import static org.globalart.astrum.security.BreadActions.BROWSE;
import org.springframework.stereotype.Component;

/**
 * @since 3.6
 */
@Component
@Path(RepositoryBrowseResource.RESOURCE_URI)
@Produces(TEXT_HTML)
public class RepositoryBrowseResource
    implements Resource
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String REPOSITORY_PATH_SEGMENT = "{noop: (/)?}{repositoryPath: ((?<=/).*)?}";

  public static final String RESOURCE_URI = "/repository/browse/{repositoryName}" + REPOSITORY_PATH_SEGMENT;

  private static final String TEMPLATE_RESOURCE = "browseContentHtml.vm";

  private final RepositoryManager repositoryManager;

  private final BrowseNodeQueryService browseNodeQueryService;

  private final BrowseNodeConfiguration configuration;

  private final TemplateHelper templateHelper;

  private final SecurityHelper securityHelper;

  private final URL template;

  @Autowired
  public RepositoryBrowseResource(
      final RepositoryManager repositoryManager,
      final BrowseNodeQueryService browseNodeQueryService,
      final BrowseNodeConfiguration configuration,
      final TemplateHelper templateHelper,
      final SecurityHelper securityHelper)
  {
    this.repositoryManager = checkNotNull(repositoryManager);
    this.browseNodeQueryService = checkNotNull(browseNodeQueryService);
    this.configuration = checkNotNull(configuration);
    this.templateHelper = checkNotNull(templateHelper);
    this.securityHelper = checkNotNull(securityHelper);
    this.template = getClass().getResource(TEMPLATE_RESOURCE);
    checkNotNull(template);
  }

  @GET
  public Response getHtml(
      @PathParam("repositoryName") final String repositoryName,
      @PathParam("repositoryPath") final String repositoryPath,
      @Context final UriInfo uriInfo)
  {
    log.debug("Get HTML directory listing for repository {} on path {}", repositoryName, repositoryPath);

    if (!uriInfo.getAbsolutePath().toString().endsWith("/")) {
      log.debug("Request does include a trailing slash, redirecting to include it");
      return Response.seeOther(UriBuilder.fromUri(uriInfo.getAbsolutePath()).path("/").build()).build();
    }

    Repository repository = repositoryManager.get(repositoryName);

    if (repository == null) {
      throw createNotFoundException(repositoryName, null);
    }

    List<String> pathSegments = new ArrayList<>();

    if (!isRoot(repositoryPath)) {
      pathSegments = asList(repositoryPath.split("/"));
    }

    Iterable<BrowseNode> browseNodes =
        browseNodeQueryService.getByPath(repository, pathSegments, configuration.getMaxHtmlNodes());

    final boolean permitted = securityHelper.allPermitted(new RepositoryViewPermission(repository, BROWSE));
    final boolean hasChildren = browseNodes != null && !Iterables.isEmpty(browseNodes);
    final List<BrowseListItem> listItems =
        hasChildren ? browseNodeQueryService.toListItems(repository, browseNodes) : Collections.emptyList();

    // if there are visible children return them, or if we are at the root node and permitted to browse the repo
    if (hasChildren || (isRoot(repositoryPath) && permitted)) {
      return Response
          .ok(templateHelper.render(template, initializeTemplateParameters(repositoryName, repositoryPath, listItems)))
          .build();
    }

    throw createNotFoundException(repositoryName, permitted ? repositoryPath : null);
  }

  private WebApplicationException createNotFoundException(final String repositoryName, final String repositoryPath) {
    if (repositoryPath == null) {
      log.debug("Requested repository could not be located or user does not have permission: {} ", repositoryName);
      return new WebApplicationException("Repository not found", NOT_FOUND);
    }
    else {
      log.debug("Requested path {} could not be located in repository {}", repositoryPath, repositoryName);
      return new WebApplicationException("Path not found", NOT_FOUND);
    }
  }

  private TemplateParameters initializeTemplateParameters(
      final String repositoryName,
      final String path,
      final List<BrowseListItem> listItems)
  {
    TemplateParameters templateParameters = templateHelper.parameters();

    if (isRoot(path)) {
      templateParameters.set("root", true);
    }

    templateParameters.set("requestPath", "/" + path);
    templateParameters.set("listItems", listItems);

    templateParameters.set("showMoreContentWarning", configuration.getMaxHtmlNodes() == listItems.size());
    if (Strings2.isBlank(path)) {
      templateParameters.set("encodedPath", String.format("/#browse/browse:%s", repositoryName));
    }
    else {
      String encodedPath = urlEncode("/" + path + "/");
      templateParameters.set("encodedPath", String.format("/#browse/browse:%s:%s", repositoryName, encodedPath));
    }

    return templateParameters;
  }

  private boolean isRoot(final String path) {
    return Strings2.isBlank(path);
  }
}
