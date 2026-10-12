package org.globalart.astrum.repository.gitlfs.internal;

import java.util.ArrayList;
import java.util.List;
import java.util.Locale;

import javax.annotation.Nullable;

import com.fasterxml.jackson.databind.JsonNode;

/**
 * A request of the batch API: an operation and the objects it applies to.
 */
public final class GitLfsBatchRequest
{
  public static final int MAX_OBJECTS = 1000;

  public static final String BASIC_TRANSFER = "basic";

  public enum Operation
  {
    DOWNLOAD,
    UPLOAD
  }

  /**
   * An object named by a request. Values are kept as sent so that invalid ones can be reported per object.
   */
  public record ObjectSpec(@Nullable String oid, long size)
  {
  }

  private final Operation operation;

  private final List<ObjectSpec> objects;

  private GitLfsBatchRequest(final Operation operation, final List<ObjectSpec> objects) {
    this.operation = operation;
    this.objects = List.copyOf(objects);
  }

  public Operation getOperation() {
    return operation;
  }

  public List<ObjectSpec> getObjects() {
    return objects;
  }

  /**
   * @throws GitLfsException when the request cannot be answered at all: an unknown operation, a hash algorithm other
   *                         than SHA-256, no transfer adapter in common, or too many objects
   */
  public static GitLfsBatchRequest parse(final JsonNode json) {
    String operationName = GitLfsJson.text(json, "operation");
    if (operationName == null) {
      throw GitLfsException.unprocessable("Missing operation");
    }
    Operation operation;
    try {
      operation = Operation.valueOf(operationName.toUpperCase(Locale.ROOT));
    }
    catch (IllegalArgumentException e) {
      throw GitLfsException.unprocessable("Unsupported operation: " + operationName);
    }

    String hashAlgorithm = GitLfsJson.text(json, "hash_algo");
    if (hashAlgorithm != null && !GitLfsObjects.HASH_ALGORITHM.equals(hashAlgorithm)) {
      throw new GitLfsException(409, "Unsupported hash algorithm: " + hashAlgorithm);
    }

    JsonNode transfers = json.get("transfers");
    if (transfers != null && transfers.isArray() && !transfers.isEmpty()) {
      boolean basic = false;
      for (JsonNode transfer : transfers) {
        basic |= BASIC_TRANSFER.equals(transfer.asText());
      }
      if (!basic) {
        throw GitLfsException.unprocessable("Only the basic transfer adapter is supported");
      }
    }

    JsonNode objectsNode = json.get("objects");
    if (objectsNode == null || !objectsNode.isArray()) {
      throw GitLfsException.unprocessable("Missing objects");
    }
    if (objectsNode.size() > MAX_OBJECTS) {
      throw new GitLfsException(413, "A batch request can name at most " + MAX_OBJECTS + " objects");
    }
    List<ObjectSpec> objects = new ArrayList<>();
    for (JsonNode object : objectsNode) {
      JsonNode size = object.get("size");
      objects.add(new ObjectSpec(GitLfsJson.text(object, "oid"),
          size != null && size.canConvertToLong() && size.isIntegralNumber() ? size.asLong() : -1));
    }
    return new GitLfsBatchRequest(operation, objects);
  }
}
