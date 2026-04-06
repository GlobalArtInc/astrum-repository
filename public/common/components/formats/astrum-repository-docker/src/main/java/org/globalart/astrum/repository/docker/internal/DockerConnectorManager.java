package org.globalart.astrum.repository.docker.internal;

import java.util.Map;
import java.util.Optional;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.TimeUnit;

import javax.annotation.Nullable;

import org.globalart.astrum.bootstrap.entrypoint.jetty.JettyServer;
import org.globalart.astrum.bootstrap.jetty.ConnectorConfiguration;

import jakarta.annotation.PreDestroy;
import org.eclipse.jetty.http.HttpScheme;
import org.eclipse.jetty.server.HttpConfiguration;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.ObjectProvider;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
public class DockerConnectorManager
{
  private static final Logger log = LoggerFactory.getLogger(DockerConnectorManager.class);

  private static final int MAX_ATTEMPTS = 120;

  private static final long RETRY_SECONDS = 5;

  private final ObjectProvider<JettyServer> jettyServer;

  private final Map<String, Registration> registrations = new ConcurrentHashMap<>();

  private final Map<Integer, String> owners = new ConcurrentHashMap<>();

  private ScheduledExecutorService retries;

  @Autowired
  public DockerConnectorManager(final ObjectProvider<JettyServer> jettyServer) {
    this.jettyServer = checkNotNull(jettyServer);
  }

  @Nullable
  public String repositoryForPort(final int port) {
    String owner = owners.get(port);
    if (owner == null) {
      return null;
    }
    Registration registration = registrations.get(key(owner, port));
    return registration != null && registration.started ? owner : null;
  }

  public Optional<String> portOwner(final int port) {
    return Optional.ofNullable(owners.get(port));
  }

  public boolean isApplicationPort(final int port) {
    try {
      JettyServer server = jettyServer.getIfAvailable();
      return server != null && server.defaultConnectors().stream().anyMatch(c -> c.getPort() == port);
    }
    catch (RuntimeException e) {
      return false;
    }
  }

  public synchronized void register(final String repositoryName, final HttpScheme scheme, final int port) {
    String owner = owners.get(port);
    if (owner != null && !owner.equals(repositoryName)) {
      log.warn("Docker connector port {} requested by {} is already used by {}", port, repositoryName, owner);
      return;
    }
    if (registrations.containsKey(key(repositoryName, port))) {
      return;
    }
    Registration registration = new Registration(repositoryName, scheme, port);
    registrations.put(key(repositoryName, port), registration);
    owners.put(port, repositoryName);
    start(registration, 1);
  }

  public synchronized void unregisterAll(final String repositoryName) {
    registrations.values().removeIf(registration -> {
      if (!registration.repositoryName.equals(repositoryName)) {
        return false;
      }
      registration.cancelled = true;
      owners.remove(registration.port, repositoryName);
      if (registration.started && registration.server != null) {
        try {
          registration.server.removeCustomConnector(registration);
          log.info("Removed docker connector {}:{} of repository {}", registration.scheme, registration.port,
              repositoryName);
        }
        catch (RuntimeException e) {
          log.warn("Unable to remove docker connector {} of repository {}", registration.port, repositoryName, e);
        }
      }
      return true;
    });
  }

  private void start(final Registration registration, final int attempt) {
    synchronized (this) {
      if (registration.cancelled || registration.started) {
        return;
      }
      JettyServer server = jettyServer.getIfAvailable();
      try {
        if (server == null) {
          throw new IllegalStateException("Jetty is not available yet");
        }
        server.addCustomConnector(registration);
        registration.server = server;
        registration.started = true;
        log.info("Started docker connector {}:{} for repository {}", registration.scheme, registration.port,
            registration.repositoryName);
        return;
      }
      catch (NullPointerException | IllegalStateException e) {
        if (e instanceof IllegalStateException && e.getMessage() != null && e.getMessage().contains("HTTPS")) {
          log.warn("Unable to start docker {} connector on port {} for {}: {}", registration.scheme,
              registration.port, registration.repositoryName, e.getMessage());
          return;
        }
        if (attempt >= MAX_ATTEMPTS) {
          log.warn("Giving up starting docker connector {} for {}", registration.port, registration.repositoryName, e);
          return;
        }
        log.debug("Jetty not ready for docker connector {} of {}, retrying", registration.port,
            registration.repositoryName);
      }
      catch (RuntimeException e) {
        log.warn("Unable to start docker connector {} for {}", registration.port, registration.repositoryName, e);
        return;
      }
      executor().schedule(() -> start(registration, attempt + 1), RETRY_SECONDS, TimeUnit.SECONDS);
    }
  }

  private synchronized ScheduledExecutorService executor() {
    if (retries == null) {
      retries = Executors.newSingleThreadScheduledExecutor(runnable -> {
        Thread thread = new Thread(runnable, "docker-connector-starter");
        thread.setDaemon(true);
        return thread;
      });
    }
    return retries;
  }

  @PreDestroy
  public synchronized void shutdown() {
    if (retries != null) {
      retries.shutdownNow();
    }
  }

  private static String key(final String repositoryName, final int port) {
    return repositoryName + ":" + port;
  }

  private static final class Registration
      implements ConnectorConfiguration
  {
    private final String repositoryName;

    private final HttpScheme scheme;

    private final int port;

    private volatile boolean started;

    private volatile JettyServer server;

    private volatile boolean cancelled;

    private Registration(final String repositoryName, final HttpScheme scheme, final int port) {
      this.repositoryName = repositoryName;
      this.scheme = scheme;
      this.port = port;
    }

    @Override
    public HttpScheme getScheme() {
      return scheme;
    }

    @Override
    public int getPort() {
      return port;
    }

    @Override
    public HttpConfiguration customize(final HttpConfiguration configuration) {
      return configuration;
    }

    @Override
    public String toString() {
      return "DockerConnector{" + repositoryName + " " + scheme + ":" + port + "}";
    }
  }
}
