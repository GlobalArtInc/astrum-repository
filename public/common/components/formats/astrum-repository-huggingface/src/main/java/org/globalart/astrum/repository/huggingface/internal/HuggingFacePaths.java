package org.globalart.astrum.repository.huggingface.internal;

import java.util.List;
import java.util.Optional;
import java.util.Set;
import java.util.regex.Pattern;
import java.util.stream.Collectors;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.huggingface.internal.HuggingFaceRequest.Kind;

import com.google.common.base.Joiner;
import com.google.common.base.Splitter;
import com.google.common.escape.Escaper;
import com.google.common.net.UrlEscapers;

/**
 * Request paths of the Hugging Face Hub protocol, the asset paths they are stored under and the remote URLs they are
 * fetched from.
 */
public final class HuggingFacePaths
{
  public static final String DEFAULT_REVISION = "main";

  private static final Pattern COMMIT = Pattern.compile("[0-9a-f]{40}");

  private static final Pattern SHA256 = Pattern.compile("[0-9a-f]{64}");

  private static final Pattern REPO_SEGMENT = Pattern.compile("[A-Za-z0-9_.\\-]{1,96}");

  private static final String API = "api";

  private static final String RESOLVE = "resolve";

  private static final String REVISION = "revision";

  private static final String TREE = "tree";

  private static final String PATHS_INFO = "paths-info";

  private static final String REFS = "refs";

  private static final Set<String> SPECIAL_REFS = Set.of("pr", "convert");

  private static final Set<String> API_KEYWORDS = Set.of(REVISION, TREE, PATHS_INFO);

  private static final Escaper SEGMENT = UrlEscapers.urlPathSegmentEscaper();

  private static final Joiner SLASH = Joiner.on('/');

  private HuggingFacePaths() {
  }

  public static boolean isCommit(@Nullable final String revision) {
    return revision != null && COMMIT.matcher(revision).matches();
  }

  public static boolean isSha256(@Nullable final String value) {
    return value != null && SHA256.matcher(value).matches();
  }

  public static Optional<HuggingFaceRequest> parse(@Nullable final String path) {
    if (path == null || !path.startsWith("/") || path.length() < 2) {
      return Optional.empty();
    }
    String trimmed = path.endsWith("/") ? path.substring(1, path.length() - 1) : path.substring(1);
    List<String> segments = Splitter.on('/').splitToList(trimmed);
    if (segments.isEmpty() || segments.contains("")) {
      return Optional.empty();
    }
    if (API.equals(segments.get(0))) {
      return parseApi(segments.subList(1, segments.size()), path);
    }
    Optional<HuggingFaceRequest> file = parseResolve(segments, path);
    if (file.isPresent()) {
      return file;
    }
    return parseStored(segments, path);
  }

  private static Optional<HuggingFaceRequest> parseApi(final List<String> rest, final String path) {
    if (rest.size() < 2) {
      return Optional.empty();
    }
    Optional<RepoType> type = RepoType.fromPlural(rest.get(0));
    if (type.isEmpty()) {
      return Optional.empty();
    }
    List<String> tail = rest.subList(1, rest.size());
    int keyword = keywordIndex(tail, API_KEYWORDS);
    if (keyword < 0) {
      if (tail.size() <= 2 && isRepoId(tail)) {
        return Optional.of(new HuggingFaceRequest(Kind.INFO, type.get(), SLASH.join(tail), null, path));
      }
      return Optional.empty();
    }
    List<String> repo = tail.subList(0, keyword);
    List<String> after = tail.subList(keyword + 1, tail.size());
    if (!isRepoId(repo) || after.isEmpty()) {
      return Optional.empty();
    }
    String repoId = SLASH.join(repo);
    switch (tail.get(keyword)) {
      case REVISION: {
        String revision = SLASH.join(after);
        return isRevision(revision)
            ? Optional.of(new HuggingFaceRequest(Kind.INFO, type.get(), repoId, revision, path))
            : Optional.empty();
      }
      case TREE: {
        int consumed = revisionLength(after);
        String revision = SLASH.join(after.subList(0, consumed));
        String directory = SLASH.join(after.subList(consumed, after.size()));
        if (!isRevision(revision) || (!directory.isEmpty() && !isFilePath(directory))) {
          return Optional.empty();
        }
        return Optional.of(new HuggingFaceRequest(Kind.TREE, type.get(), repoId, revision, directory));
      }
      default: {
        int consumed = revisionLength(after);
        String revision = SLASH.join(after.subList(0, consumed));
        if (consumed != after.size() || !isRevision(revision)) {
          return Optional.empty();
        }
        return Optional.of(new HuggingFaceRequest(Kind.PATHS_INFO, type.get(), repoId, revision, path));
      }
    }
  }

  private static Optional<HuggingFaceRequest> parseResolve(final List<String> segments, final String path) {
    RepoType type = RepoType.MODEL;
    int start = 0;
    String first = segments.get(0);
    if (RepoType.DATASET.plural().equals(first) || RepoType.SPACE.plural().equals(first)) {
      type = RepoType.fromPlural(first).orElseThrow();
      start = 1;
    }
    else if (RepoType.MODEL.plural().equals(first)) {
      return Optional.empty();
    }
    List<String> rest = segments.subList(start, segments.size());
    int keyword = keywordIndex(rest, Set.of(RESOLVE));
    if (keyword < 0) {
      return Optional.empty();
    }
    List<String> repo = rest.subList(0, keyword);
    List<String> after = rest.subList(keyword + 1, rest.size());
    if (!isRepoId(repo) || after.size() < 2) {
      return Optional.empty();
    }
    int consumed = revisionLength(after);
    String revision = SLASH.join(after.subList(0, consumed));
    String file = SLASH.join(after.subList(consumed, after.size()));
    if (!isRevision(revision) || !isFilePath(file)) {
      return Optional.empty();
    }
    return Optional.of(new HuggingFaceRequest(Kind.FILE, type, SLASH.join(repo), revision, file));
  }

  private static Optional<HuggingFaceRequest> parseStored(final List<String> segments, final String path) {
    Optional<RepoType> type = RepoType.fromPlural(segments.get(0));
    if (type.isEmpty()) {
      return Optional.empty();
    }
    List<String> rest = segments.subList(1, segments.size());
    for (int commit : new int[]{2, 1}) {
      if (rest.size() > commit + 1 && isCommit(rest.get(commit)) && isRepoId(rest.subList(0, commit))) {
        String file = SLASH.join(rest.subList(commit + 1, rest.size()));
        if (isFilePath(file)) {
          return Optional.of(new HuggingFaceRequest(Kind.FILE, type.get(), SLASH.join(rest.subList(0, commit)),
              rest.get(commit), file));
        }
      }
    }
    return Optional.of(HuggingFaceRequest.stored(path));
  }

  private static int keywordIndex(final List<String> segments, final Set<String> keywords) {
    if (segments.size() > 2 && keywords.contains(segments.get(2))) {
      return 2;
    }
    if (segments.size() > 1 && keywords.contains(segments.get(1))) {
      return 1;
    }
    return -1;
  }

  private static int revisionLength(final List<String> segments) {
    if (segments.size() >= 3 && REFS.equals(segments.get(0)) && SPECIAL_REFS.contains(segments.get(1))) {
      return 3;
    }
    return 1;
  }

  static boolean isRepoId(final List<String> segments) {
    if (segments.isEmpty() || segments.size() > 2) {
      return false;
    }
    return segments.stream().allMatch(segment -> REPO_SEGMENT.matcher(segment).matches() && !isDots(segment));
  }

  static boolean isRevision(final String revision) {
    if (revision.isEmpty() || revision.length() > 255) {
      return false;
    }
    return Splitter.on('/').splitToList(revision).stream()
        .allMatch(segment -> !segment.isEmpty() && !isDots(segment) && segment.chars().noneMatch(Character::isISOControl));
  }

  static boolean isFilePath(final String file) {
    if (file.isEmpty() || file.length() > 1024) {
      return false;
    }
    return Splitter.on('/').splitToList(file).stream()
        .allMatch(segment -> !segment.isEmpty() && !isDots(segment) && segment.chars().noneMatch(Character::isISOControl));
  }

  private static boolean isDots(final String segment) {
    return ".".equals(segment) || "..".equals(segment);
  }

  public static String revisionPath(final RepoType type, final String repoId, final String revision) {
    return "/" + type.plural() + "/" + repoId + "/" + revision + ".json";
  }

  public static String filePath(final RepoType type, final String repoId, final String commit, final String file) {
    return "/" + type.plural() + "/" + repoId + "/" + commit + "/" + file;
  }

  public static String revisionUrl(final RepoType type, final String repoId, final String revision) {
    return API + "/" + type.plural() + "/" + repoId + "/" + REVISION + "/" + SEGMENT.escape(revision) + "?blobs=true";
  }

  public static String fileUrl(final RepoType type, final String repoId, final String revision, final String file) {
    return type.urlPrefix() + repoId + "/" + RESOLVE + "/" + SEGMENT.escape(revision) + "/" + escapePath(file);
  }

  private static String escapePath(final String file) {
    return Splitter.on('/').splitToList(file).stream().map(SEGMENT::escape).collect(Collectors.joining("/"));
  }
}
