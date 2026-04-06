package org.globalart.astrum.repository.docker.internal;

import java.util.HashSet;
import java.util.Optional;
import java.util.Set;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.docker.DockerConfig;
import org.globalart.astrum.repository.docker.DockerFormat;
import org.globalart.astrum.validation.ConstraintViolationFactory;

import jakarta.validation.ConstraintViolation;
import org.eclipse.jetty.http.HttpScheme;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.validation.ConstraintViolations.maybePropagate;

@Facet.Exposed
@Component
@Qualifier(DockerFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class DockerConnectorFacet
    extends FacetSupport
{
  private final DockerConnectorManager connectorManager;

  private final ConstraintViolationFactory constraintViolationFactory;

  private Integer httpPort;

  private Integer httpsPort;

  @Autowired
  public DockerConnectorFacet(
      final DockerConnectorManager connectorManager,
      final ConstraintViolationFactory constraintViolationFactory)
  {
    this.connectorManager = checkNotNull(connectorManager);
    this.constraintViolationFactory = checkNotNull(constraintViolationFactory);
  }

  @Override
  protected void doValidate(final Configuration configuration) throws Exception {
    Set<ConstraintViolation<?>> violations = new HashSet<>();
    validatePort(configuration, DockerConfig.HTTP_PORT, DockerConfig.httpPort(configuration), violations);
    validatePort(configuration, DockerConfig.HTTPS_PORT, DockerConfig.httpsPort(configuration), violations);
    Integer http = DockerConfig.httpPort(configuration);
    if (http != null && http.equals(DockerConfig.httpsPort(configuration))) {
      violations.add(constraintViolationFactory.createViolation("attributes.docker.httpsPort",
          "HTTP and HTTPS connector ports must differ"));
    }
    maybePropagate(violations, log);
  }

  private void validatePort(
      final Configuration configuration,
      final String key,
      final Integer port,
      final Set<ConstraintViolation<?>> violations)
  {
    if (port == null) {
      return;
    }
    String path = "attributes.docker." + key;
    if (port < 1 || port > 65535) {
      violations.add(constraintViolationFactory.createViolation(path, "Port must be between 1 and 65535"));
      return;
    }
    Optional<String> owner = connectorManager.portOwner(port);
    if (owner.isPresent() && !owner.get().equals(configuration.getRepositoryName())) {
      violations.add(constraintViolationFactory.createViolation(path,
          "Port " + port + " is already used by repository " + owner.get()));
    }
    else if (connectorManager.isApplicationPort(port)) {
      violations.add(constraintViolationFactory.createViolation(path,
          "Port " + port + " is already used by the application"));
    }
  }

  @Override
  protected void doConfigure(final Configuration configuration) throws Exception {
    httpPort = DockerConfig.httpPort(configuration);
    httpsPort = DockerConfig.httpsPort(configuration);
  }

  @Override
  protected void doStart() throws Exception {
    String repositoryName = getRepository().getName();
    if (httpPort != null) {
      connectorManager.register(repositoryName, HttpScheme.HTTP, httpPort);
    }
    if (httpsPort != null) {
      connectorManager.register(repositoryName, HttpScheme.HTTPS, httpsPort);
    }
  }

  @Override
  protected void doStop() throws Exception {
    connectorManager.unregisterAll(getRepository().getName());
  }

  @Override
  protected void doDelete() throws Exception {
    connectorManager.unregisterAll(getRepository().getName());
  }
}
