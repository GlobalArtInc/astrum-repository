package org.globalart.astrum.repository.gitlfs.internal.hosted;

import java.util.OptionalLong;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.gitlfs.internal.GitLfsBatch;
import org.globalart.astrum.repository.gitlfs.internal.GitLfsBatchRequest;
import org.globalart.astrum.repository.gitlfs.internal.GitLfsBatchRequest.Operation;
import org.globalart.astrum.repository.gitlfs.internal.GitLfsContentFacet;
import org.globalart.astrum.repository.gitlfs.internal.GitLfsException;
import org.globalart.astrum.repository.gitlfs.internal.GitLfsJson;
import org.globalart.astrum.repository.gitlfs.internal.GitLfsObjects;
import org.globalart.astrum.repository.gitlfs.internal.GitLfsPaths;
import org.globalart.astrum.repository.gitlfs.internal.security.GitLfsSecurityFacet;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.security.BreadActions;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Component;

/**
 * The batch API and the basic transfer of a hosted Git LFS repository.
 */
@Component
public class GitLfsHostedHandlers
{
  private static final Logger log = LoggerFactory.getLogger(GitLfsHostedHandlers.class);

  public final Handler batch = context -> {
    checkAccept(context.getRequest());
    GitLfsBatchRequest request = GitLfsBatchRequest.parse(GitLfsJson.read(context.getRequest().getPayload()));
    GitLfsContentFacet content = content(context);
    boolean upload = request.getOperation() == Operation.UPLOAD;
    if (upload && content.isReadOnly()) {
      throw GitLfsException.forbidden("Repository " + context.getRepository().getName() + " is read-only");
    }
    GitLfsSecurityFacet security = context.getRepository().facet(GitLfsSecurityFacet.class);
    String action = upload ? BreadActions.ADD : BreadActions.READ;
    ObjectNode response = GitLfsBatch.respond(request, GitLfsPaths.apiUrl(context), content::size,
        oid -> security.isPermitted(context.getRequest(), oid, action));
    return GitLfsJson.ok(response);
  };

  public final Handler upload = context -> {
    String oid = GitLfsPaths.matched(context).value();
    Payload payload = context.getRequest().getPayload();
    if (payload == null) {
      throw GitLfsException.badRequest("Missing object content");
    }
    GitLfsContentFacet content = content(context);
    if (content.isReadOnly()) {
      throw GitLfsException.forbidden("Repository " + context.getRepository().getName() + " is read-only");
    }
    if (content.getObject(oid).isEmpty()) {
      content.put(oid, payload);
      log.debug("Stored object {} in {}", oid, context.getRepository().getName());
    }
    return HttpResponses.ok();
  };

  public final Handler verify = context -> {
    JsonNode body = GitLfsJson.read(context.getRequest().getPayload());
    String oid = GitLfsJson.text(body, "oid");
    JsonNode size = body.get("size");
    if (!GitLfsObjects.isValidOid(oid) || size == null || !size.isIntegralNumber()) {
      throw GitLfsException.unprocessable("Verify needs the oid and size of an object");
    }
    OptionalLong stored = content(context).size(oid);
    if (stored.isEmpty()) {
      throw GitLfsException.notFound("Object " + oid + " does not exist");
    }
    if (stored.getAsLong() != size.asLong()) {
      throw GitLfsException.unprocessable(
          "Object " + oid + " has " + stored.getAsLong() + " bytes, not " + size.asLong());
    }
    ObjectNode response = GitLfsJson.object();
    response.put("oid", oid);
    response.put("size", stored.getAsLong());
    return GitLfsJson.ok(response);
  };

  public final Handler download = context -> {
    String oid = GitLfsPaths.matched(context).value();
    return content(context).getObject(oid)
        .map(asset -> HttpResponses.ok(asset.download()))
        .orElseGet(() -> HttpResponses.notFound("Object " + oid + " does not exist"));
  };

  /**
   * Refuses clients that accept neither the Git LFS media type nor JSON.
   */
  static void checkAccept(final Request request) {
    String accept = header(request, "Accept");
    if (accept != null && !accept.contains("git-lfs") && !accept.contains("json") && !accept.contains("*/*")) {
      throw new GitLfsException(406, "Responses are sent as " + GitLfsJson.MEDIA_TYPE);
    }
  }

  @Nullable
  private static String header(final Request request, final String name) {
    return request.getHeaders().names().stream()
        .filter(name::equalsIgnoreCase)
        .findFirst()
        .map(request.getHeaders()::get)
        .orElse(null);
  }

  private static GitLfsContentFacet content(final Context context) {
    Repository repository = context.getRepository();
    return repository.facet(GitLfsContentFacet.class);
  }
}
