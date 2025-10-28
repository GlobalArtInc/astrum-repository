
package org.globalart.astrum.common.node;

import java.io.InputStream;
import java.net.InetAddress;
import java.nio.charset.StandardCharsets;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionStage;
import java.util.concurrent.TimeUnit;

import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;

import org.apache.commons.io.IOUtils;

/**
 * Support class for {@link NodeAccess} to share common code.
 */
public abstract class NodeAccessSupport
    extends StateGuardLifecycleSupport
    implements NodeAccess
{
  private final CompletableFuture<String> future = CompletableFuture.supplyAsync(this::compute);

  @Override
  public CompletionStage<String> getHostName() {
    return future;
  }

  private String compute() {
    Optional<String> hostname = environmentVariable()
        .or(this::cli)
        .or(this::inetAddress)
        .map(String::trim);

    if (!hostname.isPresent()) {
      log.warn("Failed to determine hostname, using nodeId instead.");
      return getId();
    }

    return hostname.get();
  }

  /*
   * Uses the HOSTNAME envvar which is the norm for Docker and Kubernetes
   */
  private static Optional<String> environmentVariable() {
    return Optional.ofNullable(System.getenv("HOSTNAME"));
  }

  /*
   * Uses the hostname CLI tool if available
   */
  private Optional<String> cli() {
    try {
      Process process = Runtime.getRuntime().exec("hostname");
      process.waitFor(5, TimeUnit.SECONDS);
      if (process.exitValue() == 0) {
        try (InputStream in = process.getInputStream()) {
          return Optional.ofNullable(IOUtils.toString(in, StandardCharsets.UTF_8));
        }
      }
    }
    catch (Exception e) { // NOSONAR
      log.debug("Failed retrieve hostname from external process", e);
    }
    return Optional.empty();
  }

  /*
   * Uses the InetAddress
   */
  private Optional<String> inetAddress() {
    try {
      return Optional.ofNullable(InetAddress.getLocalHost().getHostName());
    }
    catch (Exception e) { // NOSONAR
      log.debug("Failed to retrieve hostname from InetAddress", e);
    }
    return Optional.empty();
  }
}
