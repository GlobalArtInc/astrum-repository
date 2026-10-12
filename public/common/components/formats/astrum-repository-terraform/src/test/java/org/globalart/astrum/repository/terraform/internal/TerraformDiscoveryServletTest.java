package org.globalart.astrum.repository.terraform.internal;

import java.io.PrintWriter;
import java.io.StringWriter;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.config.internal.ConfigurationData;
import org.globalart.astrum.repository.manager.RepositoryManager;

import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;
import org.junit.Test;

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

public class TerraformDiscoveryServletTest
{
  private final RepositoryManager repositoryManager = mock(RepositoryManager.class);

  private final TerraformDiscoveryServlet servlet = new TerraformDiscoveryServlet(repositoryManager);

  @Test
  public void answersWithTheServicesOfTheRepositoryOfTheHost() throws Exception {
    List<Repository> repositories = List.of(
        repository("tf-group", "terraform", "terraform.example.com", true),
        repository("tf-offline", "terraform", "offline.example.com", false),
        repository("npm-hosted", "npm", "terraform.example.com", true));
    when(repositoryManager.browse()).thenReturn(repositories);
    StringWriter body = new StringWriter();
    HttpServletResponse response = response(body);

    servlet.doGet(request("terraform.example.com", 443, "/astrum"), response);

    verify(response).setStatus(200);
    verify(response).setContentType("application/json");
    assertThat(body.toString()).isEqualTo("{\"modules.v1\":\"/astrum/repository/tf-group/v1/modules/\","
        + "\"providers.v1\":\"/astrum/repository/tf-group/v1/providers/\"}");
  }

  @Test
  public void offersNoServicesForOtherHosts() throws Exception {
    List<Repository> repositories = List.of(repository("tf-offline", "terraform", "offline.example.com", false));
    when(repositoryManager.browse()).thenReturn(repositories);
    StringWriter body = new StringWriter();
    HttpServletResponse response = response(body);

    servlet.doGet(request("offline.example.com", 443, ""), response);

    verify(response).setStatus(404);
    assertThat(body.toString()).contains("No Terraform repository answers for host offline.example.com");
  }

  private static HttpServletRequest request(final String host, final int port, final String contextPath) {
    HttpServletRequest request = mock(HttpServletRequest.class);
    when(request.getServerName()).thenReturn(host);
    when(request.getServerPort()).thenReturn(port);
    when(request.getContextPath()).thenReturn(contextPath);
    return request;
  }

  private static HttpServletResponse response(final StringWriter body) throws Exception {
    HttpServletResponse response = mock(HttpServletResponse.class);
    when(response.getWriter()).thenReturn(new PrintWriter(body, true));
    return response;
  }

  private static Repository repository(final String name, final String format, final String host,
                                       final boolean online)
  {
    Configuration configuration = new ConfigurationData();
    configuration.setOnline(online);
    Map<String, Map<String, Object>> attributes = new HashMap<>();
    attributes.put("terraform", new HashMap<>(Map.of("registryHost", host)));
    configuration.setAttributes(attributes);
    Repository repository = mock(Repository.class);
    when(repository.getName()).thenReturn(name);
    when(repository.getFormat()).thenReturn(new Format(format) { });
    when(repository.getConfiguration()).thenReturn(configuration);
    return repository;
  }
}
