package org.globalart.astrum.repository.huggingface.internal;

import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ArrayNode;
import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.is;
import static org.junit.jupiter.api.Assertions.assertThrows;

class RevisionInfoTest
{
  private static final String SHA = "8f4dff0307bd726d0932f44183f1a88e297bcfe4";

  private static final String LFS_SHA256 = "84d89877f0d4041efb6bf91a16f0248f2fd573e6af05c19f96bedb9f882f7882";

  private static final String JSON = "{\"id\":\"acme/tiny\",\"sha\":\"" + SHA + "\",\"private\":false,\"siblings\":["
      + "{\"rfilename\":\".gitattributes\",\"blobId\":\"104a4c771e66414a9dad5e0c2c1532da74f21636\",\"size\":18},"
      + "{\"rfilename\":\"config.json\",\"blobId\":\"cb5b2f69babc53f9145b45ad20b9a075d48717ef\",\"size\":9},"
      + "{\"rfilename\":\"onnx/model.onnx\",\"blobId\":\"936e1b4672d9b9eae2dbee72f68e5f344411832b\",\"size\":10,"
      + "\"lfs\":{\"sha256\":\"" + LFS_SHA256 + "\",\"size\":10,\"pointerSize\":128}},"
      + "{\"rfilename\":\"onnx/sub/a.txt\",\"blobId\":\"587be6b4c3f93f93c489c0111bba5596147a26cb\",\"size\":2},"
      + "{\"rfilename\":\"onnx-extra.txt\",\"blobId\":\"975fbec8256d3e8a3797e7a3611380f27c49f4ac\",\"size\":2}"
      + "]}";

  private final RevisionInfo info = RevisionInfo.parse(JSON.getBytes(StandardCharsets.UTF_8));

  @Test
  void readsTheCommitAndTheFiles() {
    assertThat(info.sha().orElseThrow(), is(SHA));
    assertThat(info.listsFiles(), is(true));
    RepoFile config = info.file("config.json").orElseThrow();
    assertThat(config.lfs(), is(false));
    assertThat(config.etag(), is("cb5b2f69babc53f9145b45ad20b9a075d48717ef"));
    assertThat(config.contentSize(), is(9L));
    RepoFile model = info.file("onnx/model.onnx").orElseThrow();
    assertThat(model.lfs(), is(true));
    assertThat(model.etag(), is(LFS_SHA256));
    assertThat(model.contentSize(), is(10L));
    assertThat(info.file("missing.bin").isPresent(), is(false));
  }

  @Test
  void keepsTheDocumentAsTheRemoteSentIt() {
    assertThat(new String(info.raw(), StandardCharsets.UTF_8), is(JSON));
  }

  @Test
  void listsTheRootWithDirectoriesFirst() {
    ArrayNode tree = info.tree("", false).orElseThrow();
    assertThat(paths(tree), contains("onnx", ".gitattributes", "config.json", "onnx-extra.txt"));
    assertThat(tree.get(0).get("type").asText(), is("directory"));
    assertThat(tree.get(2).get("type").asText(), is("file"));
    assertThat(tree.get(2).get("oid").asText(), is("cb5b2f69babc53f9145b45ad20b9a075d48717ef"));
    assertThat(tree.get(2).get("size").asLong(), is(9L));
  }

  @Test
  void listsSubdirectoriesAndRecursively() {
    ArrayNode onnx = info.tree("onnx/", false).orElseThrow();
    assertThat(paths(onnx), contains("onnx/sub", "onnx/model.onnx"));
    JsonNode lfs = onnx.get(1).get("lfs");
    assertThat(lfs.get("oid").asText(), is(LFS_SHA256));
    assertThat(lfs.get("size").asLong(), is(10L));
    assertThat(lfs.get("pointerSize").asLong(), is(128L));

    assertThat(paths(info.tree("", true).orElseThrow()), contains("onnx", "onnx/sub", ".gitattributes",
        "config.json", "onnx-extra.txt", "onnx/model.onnx", "onnx/sub/a.txt"));
    assertThat(info.tree("config.json", false).isPresent(), is(false));
    assertThat(info.tree("missing", true).isPresent(), is(false));
  }

  @Test
  void computesGitTreeIds() {
    assertThat(info.treeId("onnx/sub").orElseThrow(), is("6ca2b082c4982a05d9978c0e48bfbae57de44389"));
    assertThat(info.treeId("onnx").orElseThrow(), is("384d89a38bae6907176e250f8a8b26355e7b0db0"));
    assertThat(info.treeId("").orElseThrow(), is("f9d266b129f87c3bfac425d50ae244346b1b1cd2"));
  }

  @Test
  void answersPathInformation() {
    ArrayNode entries = info.pathsInfo(List.of("config.json", "onnx", "missing.bin", "/onnx/sub/a.txt"));
    assertThat(paths(entries), contains("config.json", "onnx", "onnx/sub/a.txt"));
    assertThat(entries.get(1).get("oid").asText(), is("384d89a38bae6907176e250f8a8b26355e7b0db0"));
  }

  @Test
  void toleratesRevisionsWithoutFileDetails() {
    RevisionInfo bare = RevisionInfo.parse(("{\"sha\":\"" + SHA + "\",\"siblings\":[{\"rfilename\":\"a/b.bin\"}]}")
        .getBytes(StandardCharsets.UTF_8));
    RepoFile file = bare.file("a/b.bin").orElseThrow();
    assertThat(file.etag() == null, is(true));
    assertThat(file.contentSize(), is(-1L));
    assertThat(bare.treeId("a").isPresent(), is(false));
    assertThat(bare.tree("a", false).orElseThrow().get(0).get("oid").asText(), is(""));

    RevisionInfo withoutFiles = RevisionInfo.parse("{\"sha\":\"abc\"}".getBytes(StandardCharsets.UTF_8));
    assertThat(withoutFiles.listsFiles(), is(false));
  }

  @Test
  void rejectsDocumentsThatAreNotRevisionInformation() {
    assertThrows(IllegalArgumentException.class, () -> RevisionInfo.parse("[]".getBytes(StandardCharsets.UTF_8)));
    assertThrows(IllegalArgumentException.class,
        () -> RevisionInfo.parse("<html>".getBytes(StandardCharsets.UTF_8)));
  }

  private static List<String> paths(final ArrayNode entries) {
    List<String> paths = new ArrayList<>();
    entries.forEach(entry -> paths.add(entry.get("path").asText()));
    return paths;
  }
}
