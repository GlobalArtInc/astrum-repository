
package org.globalart.astrum.repository.cocoapods.internal.proxy;

import java.net.URI;

import org.globalart.astrum.repository.cocoapods.internal.git.GitApiHelper;
import org.globalart.astrum.repository.cocoapods.internal.proxy.SpecFileProcessor.SpecInfo;

import org.junit.Test;

import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.CoreMatchers.nullValue;
import static org.hamcrest.MatcherAssert.assertThat;

public class SpecFileProcessorTest
{
  private static final URI REPO_URI = URI.create("http://repouri/");

  private final GitApiHelper gitApiHelper =
      new GitApiHelper("https://github.com", "https://bitbucket.org", "https://gitlab.com");

  private final SpecFileProcessor processor = new SpecFileProcessor(gitApiHelper);

  @Test
  public void gitHubToProxiedSpecPositiveTest() throws Exception {
    String spec = "{" +
        "\"name\": \"MasonryHidden\"," +
        "\"version\": \"1.0.0\"," +
        "\"source\": {\n" +
        "\"git\": \"https://github.com/SunnySunning/MasonryHidden.git\"," +
        "\"tag\": \"0.5.0\"" +
        "}" +
        "}";

    String transformedSpec = "{\n" +
        "  \"name\" : \"MasonryHidden\",\n" +
        "  \"version\" : \"1.0.0\",\n" +
        "  \"source\" : {\n" +
        "    \"http\" : \"http://repouri/pods/MasonryHidden/1.0.0/0.5.0.tar.gz\"\n" +
        "  }\n" +
        "}";

    assertThat(processor.toProxiedSpec(spec, REPO_URI), is(transformedSpec));

    SpecInfo info = processor.parse(spec);
    assertThat(info.getName(), is("MasonryHidden"));
    assertThat(info.getVersion(), is("1.0.0"));
    assertThat(info.getRemoteUri().toString(),
        is("https://github.com/SunnySunning/MasonryHidden/archive/0.5.0.tar.gz"));
  }

  @Test
  public void gitHubToProxiedSpecNumericTagTest() throws Exception {
    String spec = "{" +
        "\"name\": \"MasonryHidden\"," +
        "\"version\": \"1.0.0\"," +
        "\"source\": {\n" +
        "\"git\": \"https://github.com/SunnySunning/MasonryHidden.git\"," +
        "\"tag\": 0.01" +
        "}" +
        "}";

    assertThat(processor.parse(spec).getRemoteUri().toString(),
        is("https://github.com/SunnySunning/MasonryHidden/archive/0.01.tar.gz"));
  }

  @Test
  public void gitHubWithoutRefTest() throws Exception {
    String spec = "{\"name\": \"Pod\", \"version\": \"2.0\", \"source\": {\"git\": \"https://github.com/v/r.git\"}}";

    assertThat(processor.parse(spec).getRemoteUri().toString(), is("https://github.com/v/r/archive/HEAD.tar.gz"));
    assertThat(processor.toProxiedSpec(spec, REPO_URI).contains("\"http://repouri/pods/Pod/2.0/HEAD.tar.gz\""),
        is(true));
  }

  @Test
  public void httpToProxiedSpecTest() throws Exception {
    String spec = "{" +
        "\"name\": \"AppSpectorTVSDK\"," +
        "\"version\": \"1.0.0\"," +
        "\"source\": {\n" +
        "\"http\": \"https://github.com/appspector/ios-sdk/blob/master/AppSpectorTVSDK.zip?raw=true\"," +
        "\"sha256\": \"abc\"," +
        "\"headers\": [\"Authorization: x\"]" +
        "}" +
        "}";

    String transformedSpec = "{\n" +
        "  \"name\" : \"AppSpectorTVSDK\",\n" +
        "  \"version\" : \"1.0.0\",\n" +
        "  \"source\" : {\n" +
        "    \"http\" : \"http://repouri/pods/AppSpectorTVSDK/1.0.0/AppSpectorTVSDK.zip\",\n" +
        "    \"sha256\" : \"abc\"\n" +
        "  }\n" +
        "}";

    assertThat(processor.toProxiedSpec(spec, REPO_URI), is(transformedSpec));
  }

  @Test
  public void unsupportedGitHostStaysAsIsTest() throws Exception {
    String spec = "{" +
        "\"name\": \"MasonryHidden\"," +
        "\"version\": \"1.0.0\"," +
        "\"source\": {\n" +
        "\"git\": \"git@gitlab.gbksoft.net:gbksoft-mobile-department/ios/gbkslidemenu.git\"," +
        "\"tag\": \"0.5.0\"" +
        "}" +
        "}";

    assertThat(processor.parse(spec).getRemoteUri(), nullValue());
    assertThat(processor.toProxiedSpec(spec, REPO_URI), is(spec));
  }

  @Test
  public void gitWithSubmodulesStaysAsIsTest() throws Exception {
    String spec = "{\"name\": \"Pod\", \"version\": \"1.0\", " +
        "\"source\": {\"git\": \"https://github.com/v/r.git\", \"tag\": \"1.0\", \"submodules\": true}}";

    assertThat(processor.parse(spec).getRemoteUri(), nullValue());
    assertThat(processor.toProxiedSpec(spec, REPO_URI), is(spec));
  }

  @Test
  public void svnSourceStaysAsIsTest() throws Exception {
    String spec = "{\"name\": \"Pod\", \"version\": \"1.0\", \"source\": {\"svn\": \"https://svn.example.com/pod\"}}";

    assertThat(processor.parse(spec).getRemoteUri(), nullValue());
    assertThat(processor.toProxiedSpec(spec, REPO_URI), is(spec));
  }

  @Test(expected = InvalidSpecFileException.class)
  public void testInvalidJson() throws Exception {
    processor.toProxiedSpec("invalid_json", REPO_URI);
  }

  @Test(expected = InvalidSpecFileException.class)
  public void gitHubToProxiedSpecNoSourceTest() throws Exception {
    processor.toProxiedSpec("{\"name\": \"MasonryHidden\",\"version\": \"1.0.0\"}", REPO_URI);
  }

  @Test(expected = InvalidSpecFileException.class)
  public void gitHubToProxiedSpecNoNameTest() throws Exception {
    String spec = "{" +
        "\"version\": \"1.0.0\"," +
        "\"source\": {\n" +
        "\"git\": \"https://github.com/SunnySunning/MasonryHidden.git\"," +
        "\"tag\": \"0.5.0\"" +
        "}" +
        "}";

    processor.toProxiedSpec(spec, REPO_URI);
  }

  @Test(expected = InvalidSpecFileException.class)
  public void gitHubToProxiedSpecNoVersionTest() throws Exception {
    String spec = "{" +
        "\"name\": \"MasonryHidden\"," +
        "\"source\": {\n" +
        "\"git\": \"https://github.com/SunnySunning/MasonryHidden.git\"," +
        "\"tag\": \"0.5.0\"" +
        "}" +
        "}";

    processor.toProxiedSpec(spec, REPO_URI);
  }

  @Test(expected = InvalidSpecFileException.class)
  public void pathTraversalNameTest() throws Exception {
    processor.parse("{\"name\": \"..\", \"version\": \"1.0\", \"source\": {\"http\": \"https://example.com/a.zip\"}}");
  }

  @Test
  public void gitHubToProxiedSpecVersionIncludeNonPrintableCharactersTest() throws Exception {
    String spec = "{\n" +
        "  \"name\": \"Realm\",\n" +
        "  \"version\": \"0.92.3\\n\",\n" +
        "  \"source\": {\n" +
        "    \"git\": \"https://github.com/realm/realm-cocoa.git\",\n" +
        "    \"tag\": \"v0.92.3\"\n" +
        "  }\n" +
        "}";

    String res = processor.toProxiedSpec(spec, REPO_URI);
    assertThat(res.contains("/Realm/0.92.3/v0.92.3.tar.gz"), is(true));
  }
}
