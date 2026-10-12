package org.globalart.astrum.repository.gitlfs.internal;

import java.util.OptionalLong;
import java.util.function.Function;
import java.util.function.Predicate;

import org.globalart.astrum.repository.gitlfs.internal.GitLfsBatchRequest.ObjectSpec;
import org.globalart.astrum.repository.gitlfs.internal.GitLfsBatchRequest.Operation;

import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;

/**
 * Answers a batch request with the basic transfer: an action per object the client has to transfer, or an error per
 * object it cannot.
 */
public final class GitLfsBatch
{
  private GitLfsBatch() {
  }

  /**
   * @param request   the batch request
   * @param apiUrl    the absolute URL of the API the hrefs are built on
   * @param stored    the size of a stored object, empty when the repository does not hold it
   * @param permitted whether the user may transfer an object in the requested direction
   * @throws GitLfsException with 403 when the user may upload none of the objects
   */
  public static ObjectNode respond(
      final GitLfsBatchRequest request,
      final String apiUrl,
      final Function<String, OptionalLong> stored,
      final Predicate<String> permitted)
  {
    ObjectNode response = GitLfsJson.object();
    response.put("transfer", GitLfsBatchRequest.BASIC_TRANSFER);
    ArrayNode objects = response.putArray("objects");
    boolean anyPermitted = request.getObjects().isEmpty();
    for (ObjectSpec spec : request.getObjects()) {
      ObjectNode object = objects.addObject();
      object.put("oid", spec.oid());
      object.put("size", spec.size());
      if (!GitLfsObjects.isValidOid(spec.oid())) {
        error(object, 422, "Invalid object id");
        anyPermitted = true;
        continue;
      }
      if (spec.size() < 0) {
        error(object, 422, "Invalid object size");
        anyPermitted = true;
        continue;
      }
      boolean allowed = permitted.test(spec.oid());
      anyPermitted |= allowed;
      if (request.getOperation() == Operation.DOWNLOAD) {
        download(object, spec, apiUrl, allowed ? stored.apply(spec.oid()) : OptionalLong.empty());
      }
      else if (!allowed) {
        error(object, 403, "Not permitted to upload this object");
      }
      else {
        upload(object, spec, apiUrl, stored.apply(spec.oid()));
      }
    }
    if (request.getOperation() == Operation.UPLOAD && !anyPermitted) {
      throw GitLfsException.forbidden("Not permitted to upload to this repository");
    }
    response.put("hash_algo", GitLfsObjects.HASH_ALGORITHM);
    return response;
  }

  private static void download(
      final ObjectNode object,
      final ObjectSpec spec,
      final String apiUrl,
      final OptionalLong size)
  {
    if (size.isEmpty()) {
      error(object, 404, "Object does not exist");
      return;
    }
    object.put("size", size.getAsLong());
    object.putObject("actions").set("download", action(objectHref(apiUrl, spec.oid())));
  }

  private static void upload(
      final ObjectNode object,
      final ObjectSpec spec,
      final String apiUrl,
      final OptionalLong size)
  {
    if (size.isPresent()) {
      object.put("size", size.getAsLong());
      return;
    }
    ObjectNode actions = object.putObject("actions");
    actions.set("upload", action(objectHref(apiUrl, spec.oid())));
    actions.set("verify", action(apiUrl + GitLfsPaths.VERIFY));
  }

  public static String objectHref(final String apiUrl, final String oid) {
    return apiUrl + "/objects/" + oid;
  }

  private static ObjectNode action(final String href) {
    ObjectNode action = GitLfsJson.object();
    action.put("href", href);
    return action;
  }

  private static void error(final ObjectNode object, final int code, final String message) {
    ObjectNode error = object.putObject("error");
    error.put("code", code);
    error.put("message", message);
  }
}
