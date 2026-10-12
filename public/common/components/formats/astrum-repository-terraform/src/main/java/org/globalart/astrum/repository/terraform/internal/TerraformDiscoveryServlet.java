package org.globalart.astrum.repository.terraform.internal;

import java.io.IOException;
import java.util.Optional;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.terraform.TerraformFormat;
import org.globalart.astrum.repository.terraform.TerraformPaths;

import com.fasterxml.jackson.databind.node.ObjectNode;
import jakarta.servlet.annotation.WebServlet;
import jakarta.servlet.http.HttpServlet;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.view.ContentTypes.APPLICATION_JSON;

/**
 * Service discovery at the root of a host, {@code /.well-known/terraform.json}, which is the only place Terraform
 * looks for it. A Terraform repository whose registry host is the host of the request answers it, so a
 * configuration addresses its modules as {@code <host>/<namespace>/<name>/<system>} and its providers as
 * {@code <host>/<namespace>/<type>}. Without such a repository the host offers no Terraform services.
 */
@WebServlet(urlPatterns = TerraformPaths.DISCOVERY)
@Component
public class TerraformDiscoveryServlet
    extends HttpServlet
{
  private final RepositoryManager repositoryManager;

  @Autowired
  public TerraformDiscoveryServlet(final RepositoryManager repositoryManager) {
    this.repositoryManager = checkNotNull(repositoryManager);
  }

  @Override
  protected void doGet(final HttpServletRequest request, final HttpServletResponse response) throws IOException {
    Optional<Repository> repository = repositoryFor(request.getServerName(), request.getServerPort());
    String body;
    if (repository.isEmpty()) {
      response.setStatus(HttpServletResponse.SC_NOT_FOUND);
      ObjectNode error = TerraformResponses.object();
      error.putArray("errors").add("No Terraform repository answers for host " + request.getServerName()
          + "; set the registry host of a Terraform repository to it");
      body = error.toString();
    }
    else {
      String base = (request.getContextPath() == null ? "" : request.getContextPath()) + "/repository/"
          + repository.get().getName();
      response.setStatus(HttpServletResponse.SC_OK);
      body = TerraformResponses.discovery(base + TerraformPaths.MODULES, base + TerraformPaths.PROVIDERS).toString();
    }
    response.setContentType(APPLICATION_JSON);
    response.setCharacterEncoding("UTF-8");
    response.setHeader("Cache-Control", "no-cache");
    response.getWriter().write(body);
  }

  @Override
  protected void doHead(final HttpServletRequest request, final HttpServletResponse response) throws IOException {
    doGet(request, response);
  }

  Optional<Repository> repositoryFor(final String serverName, final int serverPort) {
    Repository found = null;
    for (Repository repository : repositoryManager.browse()) {
      if (!TerraformFormat.NAME.equals(repository.getFormat().getValue())
          || !repository.getConfiguration().isOnline()) {
        continue;
      }
      Optional<String> host = TerraformRepositoryConfig.registryHost(repository.getConfiguration());
      if (host.isPresent() && TerraformRepositoryConfig.matchesHost(host.get(), serverName, serverPort)
          && (found == null || repository.getName().compareTo(found.getName()) < 0)) {
        found = repository;
      }
    }
    return Optional.ofNullable(found);
  }
}
