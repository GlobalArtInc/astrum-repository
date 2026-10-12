package org.globalart.astrum.repository.huggingface.internal;

import java.util.Optional;

import org.globalart.astrum.repository.huggingface.internal.HuggingFaceRequest.Kind;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;

class HuggingFacePathsTest
{
  private static final String COMMIT = "0123456789abcdef0123456789abcdef01234567";

  @Test
  void parsesModelFileUrls() {
    HuggingFaceRequest request = parse("/openai-community/gpt2/resolve/main/config.json");
    assertThat(request.kind(), is(Kind.FILE));
    assertThat(request.type(), is(RepoType.MODEL));
    assertThat(request.repoId(), is("openai-community/gpt2"));
    assertThat(request.revision(), is("main"));
    assertThat(request.path(), is("config.json"));
  }

  @Test
  void parsesRepositoriesWithoutNamespace() {
    HuggingFaceRequest request = parse("/gpt2/resolve/" + COMMIT + "/onnx/decoder_model.onnx");
    assertThat(request.repoId(), is("gpt2"));
    assertThat(request.revision(), is(COMMIT));
    assertThat(request.path(), is("onnx/decoder_model.onnx"));
  }

  @Test
  void parsesDatasetAndSpaceFileUrls() {
    HuggingFaceRequest dataset = parse("/datasets/stanfordnlp/imdb/resolve/main/plain_text/train.parquet");
    assertThat(dataset.type(), is(RepoType.DATASET));
    assertThat(dataset.repoId(), is("stanfordnlp/imdb"));
    assertThat(dataset.path(), is("plain_text/train.parquet"));

    HuggingFaceRequest space = parse("/spaces/acme/demo/resolve/main/app.py");
    assertThat(space.type(), is(RepoType.SPACE));
    assertThat(space.repoId(), is("acme/demo"));
  }

  @Test
  void keepsPullRequestAndConversionRefsTogether() {
    HuggingFaceRequest file = parse("/acme/model/resolve/refs/pr/12/model.safetensors");
    assertThat(file.revision(), is("refs/pr/12"));
    assertThat(file.path(), is("model.safetensors"));

    HuggingFaceRequest tree = parse("/api/datasets/acme/data/tree/refs/convert/parquet/default");
    assertThat(tree.kind(), is(Kind.TREE));
    assertThat(tree.revision(), is("refs/convert/parquet"));
    assertThat(tree.path(), is("default"));
  }

  @Test
  void parsesRevisionInformation() {
    HuggingFaceRequest latest = parse("/api/models/acme/model");
    assertThat(latest.kind(), is(Kind.INFO));
    assertThat(latest.repoId(), is("acme/model"));
    assertThat(latest.revision(), is(nullValue()));

    HuggingFaceRequest atBranch = parse("/api/models/acme/model/revision/v1.0");
    assertThat(atBranch.revision(), is("v1.0"));

    HuggingFaceRequest legacy = parse("/api/models/bert-base-uncased/revision/main");
    assertThat(legacy.repoId(), is("bert-base-uncased"));

    HuggingFaceRequest dataset = parse("/api/datasets/acme/data/revision/refs/pr/3");
    assertThat(dataset.type(), is(RepoType.DATASET));
    assertThat(dataset.revision(), is("refs/pr/3"));
  }

  @Test
  void parsesTreeListingsAndPathInformation() {
    HuggingFaceRequest root = parse("/api/models/acme/model/tree/main");
    assertThat(root.kind(), is(Kind.TREE));
    assertThat(root.path(), is(""));

    HuggingFaceRequest nested = parse("/api/models/acme/model/tree/" + COMMIT + "/onnx/sub/");
    assertThat(nested.revision(), is(COMMIT));
    assertThat(nested.path(), is("onnx/sub"));

    HuggingFaceRequest paths = parse("/api/models/acme/model/paths-info/main");
    assertThat(paths.kind(), is(Kind.PATHS_INFO));
    assertThat(paths.revision(), is("main"));
  }

  @Test
  void parsesStoredAssetPaths() {
    HuggingFaceRequest file = parse("/models/acme/model/" + COMMIT + "/onnx/model.onnx");
    assertThat(file.kind(), is(Kind.FILE));
    assertThat(file.repoId(), is("acme/model"));
    assertThat(file.revision(), is(COMMIT));
    assertThat(file.path(), is("onnx/model.onnx"));

    HuggingFaceRequest legacy = parse("/models/gpt2/" + COMMIT + "/config.json");
    assertThat(legacy.repoId(), is("gpt2"));

    HuggingFaceRequest info = parse("/models/acme/model/main.json");
    assertThat(info.kind(), is(Kind.STORED));
    assertThat(info.path(), is("/models/acme/model/main.json"));
  }

  @Test
  void rejectsInvalidPaths() {
    assertThat(HuggingFacePaths.parse("/"), is(Optional.empty()));
    assertThat(HuggingFacePaths.parse("/acme/model/resolve/main"), is(Optional.empty()));
    assertThat(HuggingFacePaths.parse("/acme/model/resolve/main/../secret"), is(Optional.empty()));
    assertThat(HuggingFacePaths.parse("/acme/model/resolve/main//config.json"), is(Optional.empty()));
    assertThat(HuggingFacePaths.parse("/a/b/c/resolve/main/config.json"), is(Optional.empty()));
    assertThat(HuggingFacePaths.parse("/api/models"), is(Optional.empty()));
    assertThat(HuggingFacePaths.parse("/api/collections/acme/model"), is(Optional.empty()));
    assertThat(HuggingFacePaths.parse("/api/models/acme/model/revision"), is(Optional.empty()));
    assertThat(HuggingFacePaths.parse("/api/models/a/b/c"), is(Optional.empty()));
    assertThat(HuggingFacePaths.parse("/unrelated/path"), is(Optional.empty()));
  }

  @Test
  void buildsAssetPathsAndRemoteUrls() {
    assertThat(HuggingFacePaths.revisionPath(RepoType.MODEL, "acme/model", "main"), is("/models/acme/model/main.json"));
    assertThat(HuggingFacePaths.filePath(RepoType.DATASET, "acme/data", COMMIT, "data/train.csv"),
        is("/datasets/acme/data/" + COMMIT + "/data/train.csv"));
    assertThat(HuggingFacePaths.revisionUrl(RepoType.MODEL, "acme/model", "refs/pr/1"),
        is("api/models/acme/model/revision/refs%2Fpr%2F1?blobs=true"));
    assertThat(HuggingFacePaths.fileUrl(RepoType.MODEL, "acme/model", COMMIT, "dir/my file#1.txt"),
        is("acme/model/resolve/" + COMMIT + "/dir/my%20file%231.txt"));
    assertThat(HuggingFacePaths.fileUrl(RepoType.SPACE, "acme/demo", "main", "app.py"),
        is("spaces/acme/demo/resolve/main/app.py"));
  }

  @Test
  void recognisesCommitsAndChecksums() {
    assertThat(HuggingFacePaths.isCommit(COMMIT), is(true));
    assertThat(HuggingFacePaths.isCommit("main"), is(false));
    assertThat(HuggingFacePaths.isCommit(COMMIT.toUpperCase()), is(false));
    assertThat(HuggingFacePaths.isSha256(COMMIT + COMMIT.substring(0, 24)), is(true));
    assertThat(HuggingFacePaths.isSha256(COMMIT), is(false));
  }

  private static HuggingFaceRequest parse(final String path) {
    return HuggingFacePaths.parse(path).orElseThrow(() -> new AssertionError("not parsed: " + path));
  }
}
