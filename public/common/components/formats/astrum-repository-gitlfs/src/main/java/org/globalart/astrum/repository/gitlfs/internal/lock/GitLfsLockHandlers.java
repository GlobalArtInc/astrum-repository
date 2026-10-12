package org.globalart.astrum.repository.gitlfs.internal.lock;

import java.util.List;
import java.util.Optional;
import java.util.function.Supplier;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.gitlfs.internal.GitLfsException;
import org.globalart.astrum.repository.gitlfs.internal.GitLfsJson;
import org.globalart.astrum.repository.gitlfs.internal.GitLfsPaths;
import org.globalart.astrum.repository.gitlfs.internal.lock.GitLfsLockFacet.Page;
import org.globalart.astrum.repository.gitlfs.internal.lock.GitLfsLockFacet.Result;
import org.globalart.astrum.repository.gitlfs.internal.security.GitLfsSecurityFacet;
import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Parameters;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.security.BreadActions;
import org.globalart.astrum.security.UserIdHelper;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * The file locking API: create, list, verify and release locks.
 */
@Component
public class GitLfsLockHandlers
{
  static final int MAX_PATH_LENGTH = 4096;

  private final Supplier<String> currentUser;

  @Autowired
  public GitLfsLockHandlers() {
    this(UserIdHelper::get);
  }

  GitLfsLockHandlers(final Supplier<String> currentUser) {
    this.currentUser = checkNotNull(currentUser);
  }

  public final Handler locks = context -> HttpMethods.POST.equals(context.getRequest().getAction())
      ? create(context)
      : list(context);

  public final Handler verify = context -> {
    JsonNode body = optionalBody(context);
    Page page = locks(context).browse(null, GitLfsJson.text(body, "cursor"), limit(body.path("limit").asText(null)));
    String user = user();
    ObjectNode response = GitLfsJson.object();
    ArrayNode ours = response.putArray("ours");
    ArrayNode theirs = response.putArray("theirs");
    for (GitLfsLock lock : page.locks()) {
      (user.equals(lock.owner()) ? ours : theirs).add(lock.toJson());
    }
    nextCursor(response, page);
    return GitLfsJson.ok(response);
  };

  public final Handler unlock = context -> {
    JsonNode body = optionalBody(context);
    GitLfsLockFacet locks = locks(context);
    GitLfsLock lock = locks.find(GitLfsPaths.matched(context).value())
        .orElseThrow(() -> GitLfsException.notFound("Lock does not exist"));
    if (!user().equals(lock.owner())) {
      if (!body.path("force").asBoolean(false)) {
        throw GitLfsException.forbidden("Lock on " + lock.path() + " is held by " + lock.owner());
      }
      GitLfsSecurityFacet security = context.getRepository().facet(GitLfsSecurityFacet.class);
      if (!security.isPermitted(context.getRequest(), BreadActions.DELETE)) {
        throw GitLfsException.forbidden("Releasing the lock of another user needs the delete privilege");
      }
    }
    locks.unlock(lock);
    return lockResponse(200, lock);
  };

  private Response create(final Context context) {
    JsonNode body = GitLfsJson.read(context.getRequest().getPayload());
    String path = GitLfsJson.text(body, "path");
    if (path == null || path.isBlank() || path.length() > MAX_PATH_LENGTH || path.indexOf('\0') >= 0) {
      throw GitLfsException.unprocessable("A lock needs the path of a file");
    }
    Result result = locks(context).lock(path, GitLfsJson.text(body.get("ref"), "name"), user());
    if (!result.created()) {
      ObjectNode conflict = GitLfsJson.object();
      conflict.set("lock", result.lock().toJson());
      conflict.put("message", "already created lock");
      throw new GitLfsException(409, "already created lock", conflict);
    }
    return lockResponse(201, result.lock());
  }

  private static Response list(final Context context) {
    Parameters parameters = context.getRequest().getParameters();
    String path = parameters.get("path");
    String id = parameters.get("id");
    GitLfsLockFacet locks = locks(context);
    Page page;
    if (id != null) {
      Optional<GitLfsLock> lock = locks.find(id).filter(found -> path == null || path.equals(found.path()));
      page = new Page(lock.map(List::of).orElse(List.of()), null);
    }
    else {
      page = locks.browse(path, parameters.get("cursor"), limit(parameters.get("limit")));
    }
    ObjectNode response = GitLfsJson.object();
    ArrayNode array = response.putArray("locks");
    page.locks().forEach(lock -> array.add(lock.toJson()));
    nextCursor(response, page);
    return GitLfsJson.ok(response);
  }

  static int limit(@Nullable final String value) {
    if (value == null || value.isBlank()) {
      return GitLfsLockFacet.DEFAULT_LIMIT;
    }
    try {
      int limit = Integer.parseInt(value.trim());
      if (limit < 1) {
        throw GitLfsException.unprocessable("The limit must be positive");
      }
      return Math.min(limit, GitLfsLockFacet.MAX_LIMIT);
    }
    catch (NumberFormatException e) {
      throw GitLfsException.unprocessable("The limit must be a number");
    }
  }

  private static void nextCursor(final ObjectNode response, final Page page) {
    if (page.nextCursor() != null) {
      response.put("next_cursor", page.nextCursor());
    }
  }

  private static Response lockResponse(final int status, final GitLfsLock lock) {
    ObjectNode response = GitLfsJson.object();
    response.set("lock", lock.toJson());
    return GitLfsJson.response(status, response);
  }

  private static JsonNode optionalBody(final Context context) {
    return GitLfsJson.readOptional(context.getRequest().getPayload());
  }

  private String user() {
    return currentUser.get();
  }

  private static GitLfsLockFacet locks(final Context context) {
    return context.getRepository().facet(GitLfsLockFacet.class);
  }
}
