package org.globalart.astrum.repository.huggingface.internal;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.ArrayList;
import java.util.Collection;
import java.util.Comparator;
import java.util.HashMap;
import java.util.HexFormat;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.TreeSet;

import javax.annotation.Nullable;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.JsonNodeFactory;
import com.fasterxml.jackson.databind.node.ObjectNode;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * The information the Hub answers for a repository revision ({@code /api/models/<repo>/revision/<revision>}), with
 * the files of the revision. Tree listings and path information are answered from it, so a revision needs a single
 * request to the remote.
 */
public final class RevisionInfo
{
  private static final ObjectMapper MAPPER = new ObjectMapper();

  private static final JsonNodeFactory NODES = JsonNodeFactory.instance;

  private static final String FILE_MODE = "100644";

  private static final String TREE_MODE = "40000";

  private final byte[] raw;

  @Nullable
  private final String sha;

  private final boolean listsFiles;

  private final Map<String, RepoFile> files;

  private final TreeSet<String> directories = new TreeSet<>();

  private final Map<String, Optional<String>> treeIds = new HashMap<>();

  private RevisionInfo(final byte[] raw, @Nullable final String sha, final boolean listsFiles,
                       final Map<String, RepoFile> files)
  {
    this.raw = raw;
    this.sha = sha;
    this.listsFiles = listsFiles;
    this.files = files;
    for (String path : files.keySet()) {
      int slash = path.lastIndexOf('/');
      while (slash > 0) {
        directories.add(path.substring(0, slash));
        slash = path.lastIndexOf('/', slash - 1);
      }
    }
  }

  /**
   * @throws IllegalArgumentException when the bytes are not a JSON object
   */
  public static RevisionInfo parse(final byte[] raw) {
    checkNotNull(raw);
    JsonNode root;
    try {
      root = MAPPER.readTree(raw);
    }
    catch (IOException e) {
      throw new IllegalArgumentException("Revision information is not JSON", e);
    }
    if (root == null || !root.isObject()) {
      throw new IllegalArgumentException("Revision information is not a JSON object");
    }
    String sha = text(root, "sha");
    JsonNode siblings = root.get("siblings");
    Map<String, RepoFile> files = new LinkedHashMap<>();
    if (siblings != null && siblings.isArray()) {
      for (JsonNode sibling : siblings) {
        String path = text(sibling, "rfilename");
        if (path == null || path.isEmpty()) {
          continue;
        }
        JsonNode lfs = sibling.get("lfs");
        String lfsSha256 = lfs != null && lfs.isObject() ? text(lfs, "sha256") : null;
        files.put(path, new RepoFile(path, text(sibling, "blobId"), number(sibling, "size"), lfsSha256,
            lfsSha256 != null ? number(lfs, "size") : -1, lfsSha256 != null ? number(lfs, "pointerSize") : -1));
      }
    }
    return new RevisionInfo(raw, sha, siblings != null && siblings.isArray(), files);
  }

  public byte[] raw() {
    return raw;
  }

  public Optional<String> sha() {
    return Optional.ofNullable(sha);
  }

  /**
   * Whether the remote listed the files of the revision; only then can a missing file be told apart from an unknown
   * one.
   */
  public boolean listsFiles() {
    return listsFiles;
  }

  public Optional<RepoFile> file(final String path) {
    return Optional.ofNullable(files.get(path));
  }

  public Collection<RepoFile> files() {
    return files.values();
  }

  public boolean isDirectory(final String path) {
    return path.isEmpty() || directories.contains(path);
  }

  /**
   * The entries of a directory as {@code /api/models/<repo>/tree/<revision>/<directory>} lists them, or empty when
   * the directory does not exist.
   */
  public Optional<ArrayNode> tree(final String directory, final boolean recursive) {
    String dir = strip(directory);
    if (!isDirectory(dir)) {
      return Optional.empty();
    }
    String prefix = dir.isEmpty() ? "" : dir + "/";
    ArrayNode entries = NODES.arrayNode();
    directories.stream()
        .filter(candidate -> candidate.startsWith(prefix) && (recursive || isChild(prefix, candidate)))
        .forEach(candidate -> entries.add(directoryEntry(candidate)));
    files.keySet().stream()
        .filter(candidate -> candidate.startsWith(prefix) && (recursive || isChild(prefix, candidate)))
        .sorted()
        .forEach(candidate -> entries.add(fileEntry(files.get(candidate))));
    return Optional.of(entries);
  }

  /**
   * The entries of the given paths as {@code POST /api/models/<repo>/paths-info/<revision>} answers them; paths that
   * do not exist are left out.
   */
  public ArrayNode pathsInfo(final Collection<String> paths) {
    ArrayNode entries = NODES.arrayNode();
    for (String path : paths) {
      String candidate = strip(path);
      RepoFile file = files.get(candidate);
      if (file != null) {
        entries.add(fileEntry(file));
      }
      else if (!candidate.isEmpty() && directories.contains(candidate)) {
        entries.add(directoryEntry(candidate));
      }
    }
    return entries;
  }

  private ObjectNode fileEntry(final RepoFile file) {
    ObjectNode entry = NODES.objectNode();
    entry.put("type", "file");
    entry.put("oid", file.blobId() != null ? file.blobId() : "");
    entry.put("size", Math.max(file.contentSize(), 0));
    if (file.lfs()) {
      ObjectNode lfs = entry.putObject("lfs");
      lfs.put("oid", file.lfsSha256());
      lfs.put("size", Math.max(file.lfsSize(), 0));
      lfs.put("pointerSize", Math.max(file.pointerSize(), 0));
    }
    entry.put("path", file.path());
    return entry;
  }

  private ObjectNode directoryEntry(final String directory) {
    ObjectNode entry = NODES.objectNode();
    entry.put("type", "directory");
    entry.put("oid", treeId(directory).orElse(""));
    entry.put("size", 0);
    entry.put("path", directory);
    return entry;
  }

  /**
   * The git tree id of a directory, computed from the object ids of its files. Files are taken as regular,
   * non-executable files, which holds for nearly every repository on the Hub.
   */
  synchronized Optional<String> treeId(final String directory) {
    Optional<String> known = treeIds.get(directory);
    if (known != null) {
      return known;
    }
    Optional<String> computed = computeTreeId(directory);
    treeIds.put(directory, computed);
    return computed;
  }

  private Optional<String> computeTreeId(final String directory) {
    String prefix = directory.isEmpty() ? "" : directory + "/";
    List<String[]> children = new ArrayList<>();
    for (String candidate : directories) {
      if (candidate.startsWith(prefix) && isChild(prefix, candidate)) {
        Optional<String> id = treeId(candidate);
        if (id.isEmpty()) {
          return Optional.empty();
        }
        children.add(new String[]{TREE_MODE, candidate.substring(prefix.length()), id.get()});
      }
    }
    for (RepoFile file : files.values()) {
      if (file.path().startsWith(prefix) && isChild(prefix, file.path())) {
        if (!HuggingFacePaths.isCommit(file.blobId())) {
          return Optional.empty();
        }
        children.add(new String[]{FILE_MODE, file.path().substring(prefix.length()), file.blobId()});
      }
    }
    children.sort(Comparator.comparing(child -> TREE_MODE.equals(child[0]) ? child[1] + "/" : child[1]));
    ByteArrayOutputStream body = new ByteArrayOutputStream();
    HexFormat hex = HexFormat.of();
    for (String[] child : children) {
      body.writeBytes((child[0] + " " + child[1]).getBytes(StandardCharsets.UTF_8));
      body.write(0);
      body.writeBytes(hex.parseHex(child[2]));
    }
    try {
      MessageDigest sha1 = MessageDigest.getInstance("SHA-1");
      sha1.update(("tree " + body.size()).getBytes(StandardCharsets.UTF_8));
      sha1.update((byte) 0);
      sha1.update(body.toByteArray());
      return Optional.of(hex.formatHex(sha1.digest()));
    }
    catch (NoSuchAlgorithmException e) {
      throw new IllegalStateException(e);
    }
  }

  private static boolean isChild(final String prefix, final String candidate) {
    return candidate.length() > prefix.length() && candidate.indexOf('/', prefix.length()) < 0;
  }

  private static String strip(final String path) {
    String result = path == null ? "" : path;
    while (result.startsWith("/")) {
      result = result.substring(1);
    }
    while (result.endsWith("/")) {
      result = result.substring(0, result.length() - 1);
    }
    return result;
  }

  @Nullable
  private static String text(final JsonNode node, final String field) {
    JsonNode value = node.get(field);
    return value != null && value.isTextual() ? value.asText() : null;
  }

  private static long number(final JsonNode node, final String field) {
    JsonNode value = node.get(field);
    return value != null && value.canConvertToLong() ? value.asLong() : -1;
  }
}
