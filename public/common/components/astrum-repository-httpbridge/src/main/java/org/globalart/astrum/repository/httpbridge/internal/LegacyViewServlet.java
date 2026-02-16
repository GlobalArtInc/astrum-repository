
package org.globalart.astrum.repository.httpbridge.internal;

import java.io.IOException;
import java.util.List;
import java.util.Objects;
import java.util.regex.Matcher;

import jakarta.servlet.ServletException;
import jakarta.servlet.annotation.WebServlet;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.httpbridge.LegacyViewConfiguration;
import org.globalart.astrum.repository.httpbridge.LegacyViewContributor;
import org.globalart.astrum.repository.httpbridge.internal.describe.DescriptionHelper;
import org.globalart.astrum.repository.httpbridge.internal.describe.DescriptionRenderer;
import org.globalart.astrum.repository.manager.RepositoryManager;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Objects.isNull;
import static org.globalart.astrum.repository.http.HttpResponses.notFound;

/**
 * Repository view servlet for NX2 urls.
 *
 * @since 3.7
 */
@Component
@WebServlet(urlPatterns = {"/content/groups/*", "/content/repositories/*", "/content/sites/*",
    "/service/local/repositories/*", "/service/local/repo_groups/*", "/service/local/nuget/*"})
public class LegacyViewServlet
    extends ViewServlet
{
  private final List<LegacyViewContributor> legacyViewContributors;

  private final RepositoryManager repositoryManager;

  @Autowired
  public LegacyViewServlet(
      final RepositoryManager repositoryManager,
      final HttpResponseSenderSelector httpResponseSenderSelector,
      final DescriptionHelper descriptionHelper,
      final DescriptionRenderer descriptionRenderer,
      final List<LegacyViewContributor> legacyViewContributors)
  {
    super(repositoryManager, httpResponseSenderSelector, descriptionHelper, descriptionRenderer);
    this.legacyViewContributors = checkNotNull(legacyViewContributors);
    this.repositoryManager = repositoryManager;
  }

  @Override
  protected void doService(final HttpServletRequest request, final HttpServletResponse response) throws Exception {
    if (handleFormatSpecificUri(request, response)) {
      return;
    }

    super.doService(request, response);
  }

  private boolean handleFormatSpecificUri(
      final HttpServletRequest request,
      final HttpServletResponse response) throws ServletException, IOException
  {
    for (LegacyViewContributor legacyViewContributor : legacyViewContributors) {
      LegacyViewConfiguration configuration = legacyViewContributor.contribute();
      if (isFormatSpecific(request, configuration)) {
        RepositoryPath path = RepositoryPath.parse(request.getPathInfo());
        Repository repository = repositoryManager.get(path.getRepositoryName());
        if (repositoryNotFoundOrFormatNotMatched(configuration, repository)) {
          send(null, notFound(REPOSITORY_NOT_FOUND_MESSAGE), response);
          return true;
        }
      }
    }
    return false;
  }

  private boolean repositoryNotFoundOrFormatNotMatched(
      final LegacyViewConfiguration configuration,
      final Repository repository)
  {
    return isNull(repository) || !Objects.equals(configuration.getFormat(), repository.getFormat().getValue());
  }

  private boolean isFormatSpecific(final HttpServletRequest request, final LegacyViewConfiguration configuration) {
    Matcher matcher = configuration.getRequestPattern().matcher(request.getRequestURI());
    return matcher.matches();
  }
}
