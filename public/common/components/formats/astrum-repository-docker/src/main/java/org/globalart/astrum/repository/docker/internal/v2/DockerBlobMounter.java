package org.globalart.astrum.repository.docker.internal.v2;

import java.io.IOException;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.docker.DockerConfig;
import org.globalart.astrum.repository.docker.DockerContentFacet;
import org.globalart.astrum.repository.docker.DockerFormat;
import org.globalart.astrum.repository.docker.internal.security.DockerAuth;
import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.ViewFacet;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import jakarta.inject.Provider;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Mounts a blob from another Docker repository of this server.
 *
 * Names reached by path carry the repository as their first segment ({@code /v2/<repository>/<name>}), so the
 * {@code from} of a mount request can name a different repository. The blob is read through that repository with the
 * permissions of the requesting user, as a pull would read it, and copied into the target repository; a blob the user
 * may not read or the source does not have leaves the client to upload it.
 */
@Component
public class DockerBlobMounter
{
  private static final Logger log = LoggerFactory.getLogger(DockerBlobMounter.class);

  private final Provider<RepositoryManager> repositoryManager;

  @Autowired
  public DockerBlobMounter(final Provider<RepositoryManager> repositoryManager) {
    this.repositoryManager = checkNotNull(repositoryManager);
  }

  public boolean mount(final Context context, final String digest, @Nullable final String from) {
    if (from == null || !DockerDigest.isSupported(digest)) {
      return false;
    }
    Object namePrefix = context.getRequest().getAttributes().get(DockerAuth.NAME_PREFIX_ATTRIBUTE);
    if (namePrefix == null || namePrefix.toString().isEmpty()) {
      return false;
    }
    int slash = from.indexOf('/');
    if (slash <= 0) {
      return false;
    }
    String repositoryName = from.substring(0, slash);
    String name = from.substring(slash + 1);
    Repository target = context.getRepository();
    if (repositoryName.equals(target.getName()) || !DockerRouteMatcher.isValidName(name)) {
      return false;
    }
    Repository source = repositoryManager.get().get(repositoryName);
    if (source == null || !DockerFormat.NAME.equals(source.getFormat().getValue())
        || !source.getConfiguration().isOnline() || !DockerConfig.pathEnabled(source)) {
      return false;
    }
    Request request = new Request.Builder()
        .action(HttpMethods.GET)
        .path("/v2/" + name + "/blobs/" + digest)
        .build();
    Response response;
    try {
      response = source.facet(ViewFacet.class).dispatch(request);
    }
    catch (Exception e) {
      log.debug("Unable to read {} from {} to mount it into {}", digest, repositoryName, target.getName(), e);
      return false;
    }
    Payload payload = response.getPayload();
    if (!response.getStatus().isSuccessful() || payload == null) {
      close(payload);
      return false;
    }
    DockerContentFacet content = target.facet(DockerContentFacet.class);
    try (TempBlob blob = content.ingest(payload, digest)) {
      if (!digest.equals(DockerDigest.of(blob, digest))) {
        log.warn("Blob {} read from {} has a different digest, not mounting it", digest, repositoryName);
        return false;
      }
      content.saveBlob(blob, digest);
      return true;
    }
    finally {
      close(payload);
    }
  }

  private static void close(@Nullable final Payload payload) {
    if (payload == null) {
      return;
    }
    try {
      payload.close();
    }
    catch (IOException e) {
      log.debug("Unable to close a mounted blob", e);
    }
  }
}
